#!/bin/bash

# SQLx database management script
# This script helps with common database operations during development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default database URL
DATABASE_URL=${DATABASE_URL:-"postgres://kiro:kiro_dev_password@localhost:5432/kiro_dev"}

echo -e "${GREEN}SQLx Database Management Script${NC}"
echo "Database URL: $DATABASE_URL"
echo ""

# Function to check if database is accessible
check_database() {
    echo -e "${YELLOW}Checking database connectivity...${NC}"
    if psql "$DATABASE_URL" -c "SELECT 1;" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Database is accessible${NC}"
        return 0
    else
        echo -e "${RED}✗ Database is not accessible${NC}"
        return 1
    fi
}

# Function to run migrations
run_migrations() {
    echo -e "${YELLOW}Running database migrations...${NC}"
    if sqlx migrate run --database-url "$DATABASE_URL"; then
        echo -e "${GREEN}✓ Migrations completed successfully${NC}"
    else
        echo -e "${RED}✗ Migration failed${NC}"
        exit 1
    fi
}

# Function to check migration status
check_migrations() {
    echo -e "${YELLOW}Checking migration status...${NC}"
    sqlx migrate info --database-url "$DATABASE_URL"
}

# Function to prepare SQLx for offline compilation
prepare_sqlx() {
    echo -e "${YELLOW}Preparing SQLx for offline compilation...${NC}"
    if cargo sqlx prepare --database-url "$DATABASE_URL"; then
        echo -e "${GREEN}✓ SQLx prepared successfully${NC}"
    else
        echo -e "${RED}✗ SQLx preparation failed${NC}"
        exit 1
    fi
}

# Function to reset database (dangerous!)
reset_database() {
    echo -e "${RED}WARNING: This will drop and recreate the database!${NC}"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Resetting database...${NC}"
        sqlx database drop --database-url "$DATABASE_URL" -y || true
        sqlx database create --database-url "$DATABASE_URL"
        run_migrations
        echo -e "${GREEN}✓ Database reset completed${NC}"
    else
        echo "Database reset cancelled"
    fi
}

# Main menu
case "${1:-help}" in
    "check")
        check_database
        ;;
    "migrate")
        check_database && run_migrations
        ;;
    "status")
        check_database && check_migrations
        ;;
    "prepare")
        check_database && prepare_sqlx
        ;;
    "reset")
        reset_database
        ;;
    "help"|*)
        echo "Usage: $0 {check|migrate|status|prepare|reset|help}"
        echo ""
        echo "Commands:"
        echo "  check    - Check database connectivity"
        echo "  migrate  - Run pending migrations"
        echo "  status   - Show migration status"
        echo "  prepare  - Prepare SQLx for offline compilation"
        echo "  reset    - Reset database (drops all data!)"
        echo "  help     - Show this help message"
        ;;
esac