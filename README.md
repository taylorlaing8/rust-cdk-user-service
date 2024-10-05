# cf-user

## Prereqs

- Install VS Code for working in CDK in Typescript and Rust
- Install Node.js 16 LTS https://nodejs.org/en
- Install cdk cli tool globally https://docs.aws.amazon.com/cdk/v2/guide/getting_started.html
- Install AWS cli tool https://aws.amazon.com/cli/ and setup default credentials using `aws configure`.

## Install

Install CDK cli
```
npm install -g aws-cdk
```

Install cdk packages

```
npm i
```

Create a `.env` file and add the following variables:

```
CDK_DEFAULT_ACCOUNT=<AWS ACCOUNT ID>
CDK_DEFAULT_REGION=<AWS REGION>
SERVICE=cf-user
STAGE=dev
AUTHORIZER_FUNCTION_ARN=<LAMBDA AUTHORIZER ARN>
CERTIFICATE_ARN=arn:aws:acm:xxx:xxxx:certificate/xxxx
```

Create an instance profile called 'cf-dev' in aws profile config

```
[profile cf-dev]
sso_start_url = <AWS START URL>
sso_region = us-west-2
sso_account_id = <AWS ACCOUNT ID>
sso_role_name = DeveloperAccess
region = us-west-2
output = json
```

## Deploy

Step 1: Refresh AWS Credentials

```
aws sso login --profile cf-dev
```

Step 2: Build Lambda

```
npm run package
```

Step 3: Deploy the app stack

```
cdk deploy cf-user-dev-app --profile cf-dev
```