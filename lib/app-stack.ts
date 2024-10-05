import * as cdk from 'aws-cdk-lib';
import * as apigateway from 'aws-cdk-lib/aws-apigateway';
import * as backup from 'aws-cdk-lib/aws-backup';
import * as events from 'aws-cdk-lib/aws-events';
import * as ddb from 'aws-cdk-lib/aws-dynamodb';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as sns from 'aws-cdk-lib/aws-sns';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import * as actions from 'aws-cdk-lib/aws-cloudwatch-actions';
import * as subscriptions from 'aws-cdk-lib/aws-sns-subscriptions';
import * as codeDeploy from 'aws-cdk-lib/aws-codedeploy';
import * as iam from 'aws-cdk-lib/aws-iam';
import { Construct } from 'constructs';

export interface AppStackProps extends cdk.StackProps {
	stage: string;
	service: string;
	subscriptionEmail: string;
	authorizerFunctionArn: string;
	certificateArn: string;
}

export class AppStack extends cdk.Stack {
	constructor(scope: Construct, id: string, props: AppStackProps) {
		super(scope, id, props);

		// TODO: Create topic that will email errors to dev@marketlink.app
		const snsTopic = new sns.Topic(this, 'SnsTopic', {
			topicName: `${this.stackName}-alarm`,
		});

		if (this.isProdStage(props.stage)) {
			snsTopic.addSubscription(
				new subscriptions.UrlSubscription(props.subscriptionEmail)
			);
		}

		const api = this.createApiGateway(props, snsTopic);

		const usersTable = this.createUsersTable();

		if (this.isCiCdStage(props.stage)) {
			const vault = new backup.BackupVault(this, 'BackupVault', {
				backupVaultName: this.stackName,
				removalPolicy: cdk.RemovalPolicy.DESTROY,
			});

			const plan = new backup.BackupPlan(this, 'BackupPlan', {
				backupPlanName: this.stackName,
				backupVault: vault,
			});

			plan.addSelection('BackupPlanSelection', {
				resources: [
					backup.BackupResource.fromDynamoDbTable(usersTable),
				],
			});

			plan.addRule(
				new backup.BackupPlanRule({
					startWindow: cdk.Duration.hours(1),
					completionWindow: cdk.Duration.hours(3),
					scheduleExpression: events.Schedule.cron({
						// Run backup once a week at the end of the week
						minute: '0',
						hour: '0',
						weekDay: '7',
						month: '*',
						year: '*',
					}),
					moveToColdStorageAfter: cdk.Duration.days(30),
					deleteAfter: cdk.Duration.days(365),
				})
			);
		}

		// API Lambdas

		const getUser = this.createLambda(
			'GetUser',
			'cf-user_get-user',
			props,
			snsTopic
		);
		usersTable.grantReadData(getUser);

		const createUser = this.createLambda(
			'CreateUser',
			'cf-user_create-user',
			props,
			snsTopic
		);
		usersTable.grantReadData(createUser);
		usersTable.grantWriteData(createUser);

		const updateUser = this.createLambda(
			'UpdateUser',
			'cf-user_update-user',
			props,
			snsTopic
		);
		usersTable.grantReadData(updateUser);
		usersTable.grantWriteData(updateUser);

		const deleteUser = this.createLambda(
			'DeleteUser',
			'cf-user_delete-user',
			props,
			snsTopic
		);
		usersTable.grantReadData(deleteUser);
		usersTable.grantWriteData(deleteUser);

		const listUsers = this.createLambda(
			'ListUsers',
			'cf-user_list-users',
			props,
			snsTopic
		);
		usersTable.grantReadData(listUsers);

		// Routes
		const v1 = api.root.addResource('v1');
		const usersV1 = v1.addResource('users');

		usersV1.addMethod("GET", new apigateway.LambdaIntegration(listUsers));
		usersV1.addMethod('POST', new apigateway.LambdaIntegration(createUser));

		const usersIdV1 = usersV1.addResource('{userId}');
		usersIdV1.addMethod('GET', new apigateway.LambdaIntegration(getUser));
		usersIdV1.addMethod("PUT", new apigateway.LambdaIntegration(updateUser));
		usersIdV1.addMethod("DELETE", new apigateway.LambdaIntegration(deleteUser));

		if (this.isCiCdStage(props.stage)) {
			new apigateway.CfnBasePathMapping(this, 'BasePathMapping', {
				domainName: `${props.stage}-api.classifind.app`,
				restApiId: api.restApiId,
				basePath: 'user',
				stage: api.deploymentStage.stageName,
			});
		}
	}

	private createUsersTable(): ddb.Table {
		const table = new ddb.Table(this, 'UsersTable', {
			tableName: `${this.stackName}-users`,
			billingMode: ddb.BillingMode.PAY_PER_REQUEST,
			encryption: ddb.TableEncryption.AWS_MANAGED,
			pointInTimeRecovery: true,
			partitionKey: {
				name: 'PK',
				type: ddb.AttributeType.STRING,
			},
			sortKey: {
				name: 'SK',
				type: ddb.AttributeType.STRING,
			},
			removalPolicy: cdk.RemovalPolicy.DESTROY,
		});

		table.addGlobalSecondaryIndex({
			indexName: 'GSI1',
			partitionKey: {
				name: 'GSI1PK',
				type: ddb.AttributeType.STRING,
			},
			sortKey: {
				name: 'GSI1SK',
				type: ddb.AttributeType.STRING,
			},
		});

		return table;
	}

	private createAuthorizer(
		props: AppStackProps
	): apigateway.RequestAuthorizer {
		const authorizerFunction = lambda.Function.fromFunctionArn(
			this,
			'LambdaAuthorizer',
			props.authorizerFunctionArn
		);

		const authorizer = new apigateway.RequestAuthorizer(
			this,
			'Auth0Authorizer',
			{
				handler: authorizerFunction,
				authorizerName: 'Auth0Authorizer',
				identitySources: [
					apigateway.IdentitySource.header('Authorization'),
				],
				resultsCacheTtl: cdk.Duration.seconds(300),
			}
		);

		return authorizer;
	}

	private createApiGateway(
		props: AppStackProps,
		snsTopic: sns.Topic
	): apigateway.RestApi {
		const authorizer = this.createAuthorizer(props);

		const apiResourcePolicy = new iam.PolicyDocument({
			statements: [
				new iam.PolicyStatement({
					effect: iam.Effect.ALLOW,
					actions: ['execute-api:Invoke'],
					principals: [new iam.AnyPrincipal()],
					resources: ['execute-api:/*'],
				}),
				// TODO: If needed for future, this is how to deny before being authorized

				// new iam.PolicyStatement({
				// 	effect: iam.Effect.DENY,
				// 	principals: [new iam.AnyPrincipal()],
				// 	actions: ["execute-api:Invoke"],
				// 	resources: ["execute-api:/*"],
				// 	conditions: {
				// 		StringNotEquals: {
				// 			"aws:SourceVpce": props.apiGatewayVpcEndpointId,
				// 		}
				// 	}
				// })
			],
		});

		const accessLogs = new logs.LogGroup(this, 'AccessLogsLogGroup', {
			logGroupName: `/aws/api-gateway/${this.stackName}`,
			retention:
				props.stage === 'prod'
					? logs.RetentionDays.ONE_YEAR
					: logs.RetentionDays.ONE_WEEK,
			removalPolicy: cdk.RemovalPolicy.DESTROY,
		});

		const api = new apigateway.RestApi(this, 'api-gateway', {
			restApiName: this.stackName,
			endpointConfiguration: {
				types: [apigateway.EndpointType.REGIONAL],
			},
			deployOptions: {
				stageName: 'LIVE',
				tracingEnabled: true,
				accessLogDestination: new apigateway.LogGroupLogDestination(
					accessLogs
				),
				accessLogFormat: apigateway.AccessLogFormat.custom(
					`{"requestTime":"${apigateway.AccessLogField.contextRequestTime()}","requestId":"${apigateway.AccessLogField.contextRequestId()}","httpMethod":"${apigateway.AccessLogField.contextHttpMethod()}","path":"${apigateway.AccessLogField.contextPath()}","resourcePath":"${apigateway.AccessLogField.contextResourcePath()}","status":${apigateway.AccessLogField.contextStatus()},"responseLatency":${apigateway.AccessLogField.contextResponseLatency()},"xrayTraceId":"${apigateway.AccessLogField.contextXrayTraceId()}","integrationLatency":"${apigateway.AccessLogField.contextIntegrationLatency()}","integrationStatus":"${apigateway.AccessLogField.contextIntegrationStatus()}","authorizerIntegrationLatency":"${apigateway.AccessLogField.contextAuthorizerIntegrationLatency()}","sourceIp":"${apigateway.AccessLogField.contextIdentitySourceIp()}","userAgent":"${apigateway.AccessLogField.contextIdentityUserAgent()}","principalId":"${apigateway.AccessLogField.contextAuthorizerPrincipalId()}"}`
				),
			},
			defaultMethodOptions: {
				authorizer: authorizer,
			},
			policy: apiResourcePolicy,
			defaultCorsPreflightOptions: {
				allowMethods: apigateway.Cors.ALL_METHODS,
				allowOrigins: apigateway.Cors.ALL_ORIGINS,
				allowHeaders: apigateway.Cors.DEFAULT_HEADERS,
				maxAge: cdk.Duration.seconds(60),
			},
			cloudWatchRole: false,
		});

		api.addGatewayResponse('AccessDeniedGatewayResponse', {
			type: apigateway.ResponseType.ACCESS_DENIED,
			statusCode: '403',
			responseHeaders: {
				'Access-Control-Allow-Origin': "'*'",
				'Access-Control-Allow-Headers': "'*'",
				'Access-Control-Allow-Methods': "'*'",
				'Access-Control-Max-Age': "'86400'",
			},
			templates: {
				'application/json':
					'{ "ErrorMessage": "$context.authorizer.errorMessage", "ErrorCode": "$context.error.responseType", "Errors": [] }',
			},
		});

		api.addGatewayResponse('UnauthorizedGatewayResponse', {
			type: apigateway.ResponseType.UNAUTHORIZED,
			statusCode: '401',
			responseHeaders: {
				'Access-Control-Allow-Origin': "'*'",
				'Access-Control-Allow-Headers': "'*'",
				'Access-Control-Allow-Methods': "'*'",
				'Access-Control-Max-Age': "'86400'",
			},
			templates: {
				'application/json':
					'{ "ErrorMessage": "Unauthorized", "ErrorCode": "$context.error.responseType", "Errors": [] }',
			},
		});

		const apiErrors = new cloudwatch.Alarm(this, 'ApiErrors', {
			alarmName: `${this.stackName}-ApiErrors`,
			alarmDescription: '500 errors > 0',
			metric: api.metricServerError({
				statistic: 'Sum',
				period: cdk.Duration.minutes(1),
			}),
			threshold: 1,
			evaluationPeriods: 1,
			actionsEnabled: true,
			comparisonOperator:
				cloudwatch.ComparisonOperator
					.GREATER_THAN_OR_EQUAL_TO_THRESHOLD,
		});

		apiErrors.addAlarmAction(new actions.SnsAction(snsTopic));

		return api;
	}

	private createLambda(
		methodName: string,
		resourceName: string,
		props: AppStackProps,
		snsTopic: sns.Topic
	): lambda.IFunction {
		const newLambda = new lambda.Function(this, methodName, {
			functionName: `${this.stackName}-${methodName}`,
			code: lambda.Code.fromAsset(`./src/${resourceName}/target/lambda/${resourceName}/bootstrap.zip`),
			handler: 'main',
			runtime: lambda.Runtime.PROVIDED_AL2,
			architecture: lambda.Architecture.ARM_64,
			timeout: cdk.Duration.seconds(30),
			memorySize: 1024,
			environment: {
				SERVICE: props.service,
				STAGE: props.stage,
			},
			tracing: lambda.Tracing.ACTIVE,
		});

		new logs.LogGroup(this, `${methodName}LogGroup`, {
			logGroupName: `/aws/lambda/${newLambda.functionName}`,
			retention:
				props.stage === 'prod'
					? logs.RetentionDays.ONE_YEAR
					: logs.RetentionDays.ONE_WEEK,
			removalPolicy: cdk.RemovalPolicy.DESTROY,
		});

		const newAlias = new lambda.Alias(this, `${methodName}Alias`, {
			aliasName: 'LIVE',
			version: newLambda.currentVersion,
		});

		const newErrors = new cloudwatch.Alarm(this, `${methodName}Errors`, {
			alarmName: `${this.stackName}-${methodName}-errors`,
			alarmDescription: 'The latest deployment errors > 0',
			metric: newAlias.metricErrors({
				statistic: 'Sum',
				period: cdk.Duration.minutes(1),
			}),
			threshold: 1,
			evaluationPeriods: 1,
			actionsEnabled: true,
			comparisonOperator:
				cloudwatch.ComparisonOperator
					.GREATER_THAN_OR_EQUAL_TO_THRESHOLD,
		});

		newErrors.addAlarmAction(new actions.SnsAction(snsTopic));

		const lambdaDeploymentConfig = this.isProdStage(props.stage)
			? codeDeploy.LambdaDeploymentConfig.CANARY_10PERCENT_10MINUTES
			: codeDeploy.LambdaDeploymentConfig.ALL_AT_ONCE;

		new codeDeploy.LambdaDeploymentGroup(
			this,
			`${methodName}DeploymentGroup`,
			{
				alias: newAlias,
				deploymentConfig: lambdaDeploymentConfig,
				alarms: [newErrors],
			}
		);

		return newAlias;
	}

	private isCiCdStage(stage: string): boolean {
		return ['dev', 'stage', 'prod'].includes(stage);
	}

	private isProdStage(stage: string): boolean {
		return 'prod' === stage;
	}
}
