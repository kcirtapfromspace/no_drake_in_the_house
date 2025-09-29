#!/bin/bash
set -euo pipefail

# Database backup script for production
# Usage: ./scripts/backup.sh [environment] [backup-type]

ENVIRONMENT=${1:-production}
BACKUP_TYPE=${2:-full}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Load environment-specific configuration
case $ENVIRONMENT in
    "production")
        NAMESPACE="kiro-production"
        BUCKET="kiro-backups-prod"
        ;;
    "staging")
        NAMESPACE="kiro-staging"
        BUCKET="kiro-backups-staging"
        ;;
    *)
        echo "Error: Unknown environment '$ENVIRONMENT'"
        echo "Usage: $0 [production|staging] [full|incremental]"
        exit 1
        ;;
esac

echo "🗄️  Starting $BACKUP_TYPE backup for $ENVIRONMENT environment..."

# Check if kubectl is available and configured
if ! command -v kubectl &> /dev/null; then
    echo "Error: kubectl is not installed or not in PATH"
    exit 1
fi

# Check if we can access the cluster
if ! kubectl cluster-info &> /dev/null; then
    echo "Error: Cannot connect to Kubernetes cluster"
    exit 1
fi

# Check if namespace exists
if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
    echo "Error: Namespace '$NAMESPACE' does not exist"
    exit 1
fi

# Create backup job based on type
if [ "$BACKUP_TYPE" = "full" ]; then
    echo "📋 Creating full database backup..."
    BACKUP_COMMAND="pg_dump --verbose --format=custom --no-owner --no-privileges"
    BACKUP_FILE="kiro_${ENVIRONMENT}_full_${TIMESTAMP}.dump"
elif [ "$BACKUP_TYPE" = "incremental" ]; then
    echo "📋 Creating incremental backup (WAL archive)..."
    BACKUP_COMMAND="pg_basebackup --verbose --format=tar --gzip --progress"
    BACKUP_FILE="kiro_${ENVIRONMENT}_incremental_${TIMESTAMP}.tar.gz"
else
    echo "Error: Unknown backup type '$BACKUP_TYPE'"
    echo "Usage: $0 [production|staging] [full|incremental]"
    exit 1
fi

# Create backup job
cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: kiro-backup-${BACKUP_TYPE}-$(date +%s)
  namespace: $NAMESPACE
  labels:
    app: kiro-backup
    backup-type: $BACKUP_TYPE
spec:
  ttlSecondsAfterFinished: 3600
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: backup
        image: postgres:15-alpine
        command: ["/bin/sh"]
        args:
        - -c
        - |
          set -e
          echo "Starting backup process..."
          
          # Create backup
          $BACKUP_COMMAND \$DATABASE_URL > /backup/$BACKUP_FILE
          
          # Verify backup
          if [ "$BACKUP_TYPE" = "full" ]; then
            pg_restore --list /backup/$BACKUP_FILE > /backup/${BACKUP_FILE}.list
            echo "Backup verification completed"
          fi
          
          # Upload to S3 (if AWS CLI is available)
          if command -v aws &> /dev/null; then
            echo "Uploading backup to S3..."
            aws s3 cp /backup/$BACKUP_FILE s3://$BUCKET/database/
            aws s3 cp /backup/${BACKUP_FILE}.list s3://$BUCKET/database/ || true
            echo "Backup uploaded successfully"
          else
            echo "AWS CLI not available, backup stored locally only"
          fi
          
          echo "Backup process completed: $BACKUP_FILE"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: DATABASE_URL
        - name: AWS_ACCESS_KEY_ID
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: AWS_ACCESS_KEY_ID
              optional: true
        - name: AWS_SECRET_ACCESS_KEY
          valueFrom:
            secretKeyRef:
              name: kiro-secrets
              key: AWS_SECRET_ACCESS_KEY
              optional: true
        - name: AWS_DEFAULT_REGION
          value: "us-west-2"
        volumeMounts:
        - name: backup-storage
          mountPath: /backup
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "500m"
      volumes:
      - name: backup-storage
        emptyDir:
          sizeLimit: 10Gi
      securityContext:
        runAsNonRoot: true
        runAsUser: 999
        fsGroup: 999
EOF

# Wait for backup to complete
echo "⏳ Waiting for backup to complete..."
JOB_NAME=$(kubectl get jobs -n "$NAMESPACE" -l app=kiro-backup,backup-type="$BACKUP_TYPE" --sort-by=.metadata.creationTimestamp -o jsonpath='{.items[-1].metadata.name}')

if ! kubectl wait --for=condition=complete job/"$JOB_NAME" -n "$NAMESPACE" --timeout=1800s; then
    echo "❌ Backup failed or timed out"
    echo "📋 Job logs:"
    kubectl logs job/"$JOB_NAME" -n "$NAMESPACE"
    exit 1
fi

echo "✅ Backup completed successfully!"

# Show backup details
echo "🔍 Backup details:"
kubectl logs job/"$JOB_NAME" -n "$NAMESPACE" | tail -20

# Clean up old backups (keep last 7 days for full, 30 days for incremental)
if command -v aws &> /dev/null; then
    echo "🧹 Cleaning up old backups..."
    if [ "$BACKUP_TYPE" = "full" ]; then
        RETENTION_DAYS=7
    else
        RETENTION_DAYS=30
    fi
    
    aws s3 ls s3://$BUCKET/database/ | while read -r line; do
        BACKUP_DATE=$(echo "$line" | awk '{print $1}')
        BACKUP_NAME=$(echo "$line" | awk '{print $4}')
        
        if [[ "$BACKUP_NAME" == *"$BACKUP_TYPE"* ]]; then
            BACKUP_AGE=$(( ($(date +%s) - $(date -d "$BACKUP_DATE" +%s)) / 86400 ))
            if [ "$BACKUP_AGE" -gt "$RETENTION_DAYS" ]; then
                echo "Deleting old backup: $BACKUP_NAME (${BACKUP_AGE} days old)"
                aws s3 rm s3://$BUCKET/database/"$BACKUP_NAME"
            fi
        fi
    done
fi

echo "🎉 Backup process completed for $ENVIRONMENT environment"
echo "📁 Backup file: $BACKUP_FILE"