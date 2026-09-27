# /// script
# dependencies = ["boto3==1.43.103"]
# ///
"""Run with uv: roll an existing ECS service while preserving task settings."""
import argparse
import json
import re
import subprocess

import boto3
from botocore.exceptions import BotoCoreError, ClientError


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile', required=True)
    parser.add_argument('--region', default='us-east-1')
    parser.add_argument('--cluster', required=True)
    parser.add_argument('--service', choices=['Frontend', 'Backend', 'News'], required=True)
    parser.add_argument('--image', required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    if not re.fullmatch(r'\d+\.dkr\.ecr\.[a-z0-9-]+\.amazonaws\.com/ndith-(?:frontend|backend)@sha256:[a-f0-9]{64}', args.image):
        raise SystemExit('Use a digest-pinned NDITH ECR image')

    ecs = boto3.Session(profile_name=args.profile, region_name=args.region).client('ecs')

    def call(operation, **kwargs):
        try:
            return getattr(ecs, operation)(**kwargs)
        except (BotoCoreError, ClientError) as error:
            code = error.response.get('Error', {}).get('Code', 'unknown') if isinstance(error, ClientError) else type(error).__name__
            raise SystemExit(f'AWS ECS {operation} failed ({code}); credentials and task environment withheld') from None

    response = call('describe_services', cluster=args.cluster, services=[args.service])
    if response.get('failures') or len(response.get('services', [])) != 1:
        raise SystemExit('Expected one existing service')
    service = response['services'][0]
    old_arn = service['taskDefinition']
    repository = 'ndith-frontend' if args.service == 'Frontend' else 'ndith-backend'
    expected_prefix = f'{old_arn.split(":")[4]}.dkr.ecr.{args.region}.amazonaws.com/{repository}@'
    if not args.image.startswith(expected_prefix):
        raise SystemExit('Release image must match this service repository, AWS account and region')
    old = call('describe_task_definition', taskDefinition=old_arn, include=['TAGS'])
    task = old['taskDefinition']
    matches = [container for container in task['containerDefinitions'] if container['name'] == args.service]
    if len(matches) != 1:
        raise SystemExit('Expected one matching application container')
    inspection = subprocess.run(['crane', 'config', args.image], capture_output=True, text=True, timeout=90)
    if inspection.returncode:
        raise SystemExit('Could not inspect release image')
    config = json.loads(inspection.stdout)
    expected = {'ARM64': 'arm64', 'X86_64': 'amd64'}[task.get('runtimePlatform', {}).get('cpuArchitecture', 'X86_64')]
    if config.get('architecture') != expected or config.get('os') != 'linux':
        raise SystemExit('Release image architecture does not match the current task')
    receipt = {'service': args.service, 'cluster': args.cluster, 'previousTaskDefinition': old_arn,
               'previousImage': matches[0]['image'], 'image': args.image, 'desiredCount': service['desiredCount'], 'architecture': expected}
    if matches[0]['image'] == args.image:
        print(json.dumps({'unchanged': True, **receipt}, indent=2))
        return
    if not args.apply:
        print(json.dumps({'dryRun': True, **receipt}, indent=2))
        return
    matches[0]['image'] = args.image
    # The SDK schema excludes output-only fields; task environment stays in memory.
    accepted = ecs.meta.service_model.operation_model('RegisterTaskDefinition').input_shape.members
    payload = {key: value for key, value in task.items() if key in accepted}
    if old.get('tags'):
        payload['tags'] = old['tags']
    registered = call('register_task_definition', **payload)
    new_arn = registered['taskDefinition']['taskDefinitionArn']
    updated = call('update_service', cluster=args.cluster, service=args.service, taskDefinition=new_arn)
    print(json.dumps({**receipt, 'taskDefinition': new_arn,
                      'deploymentIds': [deployment['id'] for deployment in updated['service']['deployments']]}, indent=2))


if __name__ == '__main__':
    main()
