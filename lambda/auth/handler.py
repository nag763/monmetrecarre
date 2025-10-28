import os

def lambda_handler(event, context):
    if secret := os.getenv('SECRET_HEADER_VALUE'):
        return {'isAuthorized': event['headers'].get('x-origin-verify') == secret}
    else:
        return {'isAuthorized': False, 'context': {'reason': 'SECRET_HEADER_VALUE not set'}}