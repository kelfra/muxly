#!/bin/bash
set -e

# Configuration
ECR_REPOSITORY="muxly"
ECS_CLUSTER="muxly-cluster"
ECS_SERVICE="muxly-service"
AWS_REGION="us-west-2"
VERSION=$(grep '^version' Cargo.toml | sed -E 's/version = "(.*)"/\1/' | tr -d '[:space:]')

# Print details
echo "Deploying to AWS: $ECR_REPOSITORY:$VERSION"
echo "Region: $AWS_REGION"
echo "Cluster: $ECS_CLUSTER"
echo "Service: $ECS_SERVICE"

# Check AWS CLI installation
if ! command -v aws &> /dev/null; then
    echo "Error: AWS CLI not found. Please install it first."
    exit 1
fi

# Check if logged in to AWS
echo "Checking AWS authentication..."
aws sts get-caller-identity > /dev/null || { echo "Error: AWS CLI not authenticated. Run 'aws configure' first."; exit 1; }
echo "AWS authentication successful."

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found. Make sure you're running this script from the project root."
    exit 1
fi

# Build Docker image first
echo "Building Docker image..."
./scripts/docker/docker-build.sh

# Login to ECR
echo "Logging in to Amazon ECR..."
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $(aws sts get-caller-identity --query Account --output text).dkr.ecr.$AWS_REGION.amazonaws.com

# Create ECR repository if it doesn't exist
echo "Checking if ECR repository exists..."
aws ecr describe-repositories --repository-names $ECR_REPOSITORY --region $AWS_REGION || aws ecr create-repository --repository-name $ECR_REPOSITORY --region $AWS_REGION

# Tag and push image to ECR
echo "Tagging and pushing image to ECR..."
ECR_URI="$(aws sts get-caller-identity --query Account --output text).dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPOSITORY"
docker tag muxly/$ECR_REPOSITORY:$VERSION $ECR_URI:$VERSION
docker tag muxly/$ECR_REPOSITORY:$VERSION $ECR_URI:latest
docker push $ECR_URI:$VERSION
docker push $ECR_URI:latest

# Update ECS service to use the new image
echo "Updating ECS service..."
aws ecs update-service \
    --cluster $ECS_CLUSTER \
    --service $ECS_SERVICE \
    --force-new-deployment \
    --region $AWS_REGION

echo "Deployment initiated. Monitor status with:"
echo "  aws ecs describe-services --cluster $ECS_CLUSTER --services $ECS_SERVICE --region $AWS_REGION"

# Wait for deployment to complete
echo "Waiting for deployment to complete..."
aws ecs wait services-stable --cluster $ECS_CLUSTER --services $ECS_SERVICE --region $AWS_REGION

echo "Deployment completed successfully." 