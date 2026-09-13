#!/bin/bash
set -e

echo "*** Check n Run Io Server Script ***"

# 1: database_postgres
database_postgres() {
    echo "Database Postgres..."

    sudo systemctl status postgresql

    # Check Status
    if [[ ! "$PKG_CONFIG_PATH" =~ "$TESSERACT" ]]; then
        echo "WARNING: Status Not Start"
        echo " "
        sudo systemctl start postgresql
    fi

    #Run
    if [ $HAS_ERROR -eq 1 ]; then
        echo ""
        echo "ERROR: Environment verification failed!"
        echo "   Please run: source ~/.bashrc"
        echo "   Then try building again"
        exit 1
    fi

    echo "[OK] Postgres run!"
    echo ""
}

# 2: database_surreal
database_surreal() {
    echo "Database Postgres..."

    # Check Status
    if [[ ! "$PKG_CONFIG_PATH" =~ "$TESSERACT" ]]; then
        echo "WARNING: PKG_CONFIG_PATH doesn't include Tesseract"
        echo "   Add to ~/.bashrc: export PKG_CONFIG_PATH=\$TESSERACT:\$PKG_CONFIG_PATH"
    fi

    #Run
    if [ $HAS_ERROR -eq 1 ]; then
        echo ""
        echo "ERROR: Environment verification failed!"
        echo "   Please run: source ~/.bashrc"
        echo "   Then try building again"
        exit 1
    fi

    echo "[OK] Postgres run!"
    echo ""
}

# 3: database_redis
database_redis() {
    echo "Database Postgres..."

    # Check Status
    if [[ ! "$PKG_CONFIG_PATH" =~ "$TESSERACT" ]]; then
        echo "WARNING: PKG_CONFIG_PATH doesn't include Tesseract"
        echo "   Add to ~/.bashrc: export PKG_CONFIG_PATH=\$TESSERACT:\$PKG_CONFIG_PATH"
    fi

    #Run
    if [ $HAS_ERROR -eq 1 ]; then
        echo ""
        echo "ERROR: Environment verification failed!"
        echo "   Please run: source ~/.bashrc"
        echo "   Then try building again"
        exit 1
    fi

    echo "[OK] Postgres run!"
    echo ""
}


# 4: database_qdrant
database_qdrant() {
    echo "Database Postgres..."

    # Check Status
    if [[ ! "$PKG_CONFIG_PATH" =~ "$TESSERACT" ]]; then
        echo "WARNING: PKG_CONFIG_PATH doesn't include Tesseract"
        echo "   Add to ~/.bashrc: export PKG_CONFIG_PATH=\$TESSERACT:\$PKG_CONFIG_PATH"
    fi

    #Run
    if [ $HAS_ERROR -eq 1 ]; then
        echo ""
        echo "ERROR: Environment verification failed!"
        echo "   Please run: source ~/.bashrc"
        echo "   Then try building again"
        exit 1
    fi

    echo "[OK] Postgres run!"
    echo ""
}


# Main execution
main() {

    #Database Server check available, status, and run
    database_postgres
    database_surreal
    database_redis
    database_qdrant

    #Manual Check on Burn-LM and or Ollama Server
}

main "$@"
