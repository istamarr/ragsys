#!/bin/bash
set -e

echo "*** Alma 9 Build Script ***"

# Step 1: Install dependencies (one-time setup on Alma 8)
install_deps() {
    echo "Installing build dependencies..."

    # Install available packages from repo
    # sudo dnf groupinstall "Development Tools" -y
    # sudo dnf install -y \
    #     openssl-devel \
    #     clang \
    #     llvm-devel \
    #     alsa-lib-devel \
    #     systemd-devel

    # Check for custom-built leptonica and tesseract
    echo ""
    echo "Checking for Leptonica and Tesseract..."

    if [ -z "$LEPTONICA" ] || [ ! -d "$LEPTONICA" ]; then
        echo "WARNING: LEPTONICA environment variable not set or directory not found"
        echo "   Currently set to: ${LEPTONICA:-<not set>}"
        echo "   Please ensure Leptonica is built and LEPTONICA env var is set in ~/.bashrc"
    else
        echo "[OK] Leptonica found at: $LEPTONICA"
        if [ -f "$LEPTONICA/lib/pkgconfig/lept.pc" ]; then
            echo "[OK] Leptonica pkg-config found"
        else
            echo "WARNING: Leptonica pkg-config not found at $LEPTONICA/lib/pkgconfig/lept.pc"
        fi
    fi

    if [ -z "$TESSERACT" ] || [ ! -d "$TESSERACT" ]; then
        echo "WARNING: TESSERACT environment variable not set or directory not found"
        echo "   Currently set to: ${TESSERACT:-<not set>}"
        echo "   Please ensure Tesseract is built and TESSERACT env var is set in ~/.bashrc"
    else
        echo "[OK] Tesseract found at: $TESSERACT"
        if [ -f "$TESSERACT/tesseract.pc" ]; then
            echo "[OK] Tesseract pkg-config found"
        else
            echo "WARNING: Tesseract pkg-config not found at $TESSERACT/tesseract.pc"
        fi
    fi

    # Install Rust if not already installed
    if ! command -v cargo &> /dev/null; then
        echo ""
        echo "Please Install Manualy From Official Site of Rust (https://rust-lang.org/tools/install/)"
        # echo "Installing Rust..."
        # curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # source $HOME/.cargo/env
    else
        echo "[OK] Rust already installed"
    fi

    echo ""
    echo "Dependencies check complete!"
    echo ""
    echo "IMPORTANT: If any warnings above, you need to:"
    echo "   1. Build Leptonica and Tesseract from source (if not done)"
    echo "   2. Set environment variables in ~/.bashrc"
    echo "   3. Run: source ~/.bashrc"
}

# Step 2: Verify environment
verify_environment() {
    echo "Verifying build environment..."

    local HAS_ERROR=0

    # Check LIBTORCH
    if [ -z "$LIBTORCH" ] || [ ! -d "$LIBTORCH" ]; then
        echo "ERROR: LIBTORCH not set or directory not found"
        echo "   Set in ~/.bashrc: export LIBTORCH=/path/to/libtorch"
        HAS_ERROR=1
    else
        echo "[OK] LIBTORCH: $LIBTORCH"
    fi

    # Check Leptonica
    if [ -z "$LEPTONICA" ] || [ ! -d "$LEPTONICA" ]; then
        echo "ERROR: LEPTONICA not set or directory not found"
        HAS_ERROR=1
    else
        echo "[OK] LEPTONICA: $LEPTONICA"
    fi

    # Check Tesseract
    if [ -z "$TESSERACT" ] || [ ! -d "$TESSERACT" ]; then
        echo "ERROR: TESSERACT not set or directory not found"
        HAS_ERROR=1
    else
        echo "[OK] TESSERACT: $TESSERACT"
    fi

    # Check PKG_CONFIG_PATH includes custom libraries
    if [[ ! "$PKG_CONFIG_PATH" =~ "$LEPTONICA/lib/pkgconfig" ]]; then
        echo "WARNING: PKG_CONFIG_PATH doesn't include Leptonica"
        echo "   Add to ~/.bashrc: export PKG_CONFIG_PATH=\$LEPTONICA/lib/pkgconfig:\$PKG_CONFIG_PATH"
    fi

    if [[ ! "$PKG_CONFIG_PATH" =~ "$TESSERACT" ]]; then
        echo "WARNING: PKG_CONFIG_PATH doesn't include Tesseract"
        echo "   Add to ~/.bashrc: export PKG_CONFIG_PATH=\$TESSERACT:\$PKG_CONFIG_PATH"
    fi

    if [ $HAS_ERROR -eq 1 ]; then
        echo ""
        echo "ERROR: Environment verification failed!"
        echo "   Please run: source ~/.bashrc"
        echo "   Then try building again"
        exit 1
    fi

    echo "[OK] Environment verified!"
    echo ""
}

# Step 3: Setup LibTorch
setup_libtorch() {
    if [ ! -d "$LIBTORCH" ]; then
        echo "ERROR: LibTorch directory not found at: $LIBTORCH"
        echo "Please set LIBTORCH environment variable in ~/.bashrc"
        exit 1
    fi

    if [ ! -d "$LIBTORCH/lib" ]; then
        echo "ERROR: $LIBTORCH/lib not found!"
        exit 1
    fi

    echo "[OK] LibTorch found at: $LIBTORCH"
}

# Step 4: Build
build_project() {
    echo "Building project..."

    # Remove musl config if exists
    # rm -rf .cargo/config.toml || true

    # Set build environment from .bashrc
    # export LD_LIBRARY_PATH=$LIBTORCH/lib:$LEPTONICA/lib:/usr/local/lib:/usr/local/lib64:$LD_LIBRARY_PATH
    # export PKG_CONFIG_PATH=$LEPTONICA/lib/pkgconfig:$TESSERACT:$PKG_CONFIG_PATH
    # export CARGO_BUILD_JOBS=2

    # Build for GNU target
    cargo build --release --target x86_64-unknown-linux-gnu

    echo "[OK] Build complete"
}

# Step 5: Collect dependencies
collect_dependencies() {
    echo "Collecting library dependencies..."

    local BINARY="$1"
    local DEPLOY_LIB="$2"

    # Get all required .so files
    ldd "$BINARY" | grep "=>" | awk '{print $3}' | while read lib; do
        if [ -f "$lib" ]; then
            local LIBNAME=$(basename "$lib")

            # Copy if not already in deploy/lib
            if [ ! -f "$DEPLOY_LIB/$LIBNAME" ]; then
                echo "  Copying: $LIBNAME"
                cp "$lib" "$DEPLOY_LIB/" 2>/dev/null || true

                # Copy symlink targets if they exist
                if [ -L "$lib" ]; then
                    local TARGET=$(readlink -f "$lib")
                    if [ -f "$TARGET" ]; then
                        cp "$TARGET" "$DEPLOY_LIB/" 2>/dev/null || true
                    fi
                fi
            fi
        fi
    done

    # Ensure LibTorch libraries are included
    echo "  Copying LibTorch libraries..."
    cp "$LIBTORCH"/lib/*.so* "$DEPLOY_LIB/" 2>/dev/null || true

    # Copy Leptonica libraries
    if [ -d "$LEPTONICA/lib" ]; then
        echo "  Copying Leptonica libraries..."
        cp "$LEPTONICA"/lib/*.so* "$DEPLOY_LIB/" 2>/dev/null || true
    fi

    # Copy custom-built libraries from /usr/local
    echo "  Copying /usr/local libraries..."
    for lib in libsndfile libdotconf libglib-2.0 libgmodule-2.0 libgobject-2.0 libgthread-2.0; do
        find /usr/local/lib /usr/local/lib64 -name "${lib}.so*" -exec cp {} "$DEPLOY_LIB/" \; 2>/dev/null || true
    done

    echo "[OK] Dependencies collected"
}

# Step 6: Create deployment package
package_for_rhel() {
    echo "Creating RHEL 8 deployment package..."

    rm -rf deploy
    mkdir -p deploy/lib

    # Copy binary
    cp target/x86_64-unknown-linux-gnu/release/* deploy/ 2>/dev/null || true

    # Remove build artifacts, keep only executables
    find deploy/ -maxdepth 1 -type f ! -executable -delete
    find deploy/ -maxdepth 1 -name "*.d" -delete
    find deploy/ -maxdepth 1 -name "*.rlib" -delete

    # Find the main binary
    local BINARY=$(find deploy/ -maxdepth 1 -type f -executable | grep -v "\.sh$" | head -n1)

    if [ -z "$BINARY" ]; then
        echo "ERROR: No binary found in deploy/"
        exit 1
    fi

    echo "[OK] Binary found: $(basename $BINARY)"

    # Collect all dependencies
    collect_dependencies "$BINARY" "deploy/lib"

    # Create run script
    cat > deploy/gnurun.sh << 'EOF'
#!/bin/bash
# DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
# export LD_LIBRARY_PATH=$DIR/lib:$LD_LIBRARY_PATH

# Find the binary (first executable that's not a .sh file)
BINARY=$(find $DIR -maxdepth 1 -type f -executable ! -name  "*.*" -o -name ".*[^.]*" | grep -v "\.sh$" | head -n1)

if [ -z "$BINARY" ]; then
    echo "Error: No executable binary found in $DIR"
    exit 1
fi

echo "Running: $(basename $BINARY)"
exec $BINARY "$@"
EOF
    chmod +x deploy/gnurun.sh

    echo "[OK] Deployment package created"
}

# Step 7: Create tarball
create_tarball() {
    local APP_NAME=$(basename $(find deploy/ -maxdepth 1 -type f -executable | grep -v "\.sh$" | head -n1))
    local TARBALL="${APP_NAME}-rhel8.tar.gz"

    echo "Creating tarball: $TARBALL"
    tar -czf "$TARBALL" deploy/

    echo ""
    echo "=== SUCCESS! ==="
    echo "Package: $TARBALL"
    echo "Size: $(du -h $TARBALL | cut -f1)"
    echo "Libraries included: $(ls deploy/lib | wc -l) files"
    echo ""
    echo "Deploy to RHEL 8:"
    echo "  1. Copy: scp $TARBALL user@rhel8-prod:/opt/"
    echo "  2. SSH:  ssh user@rhel8-prod"
    echo "  3. Extract: cd /opt && tar -xzf $TARBALL"
    echo "  4. Run: cd deploy && ./gnurun.sh"
    echo ""
    echo "Before deploying, test locally:"
    echo "  cd deploy && ./gnurun.sh"
}

# Step 8: Verify deployment package
verify_package() {
    echo ""
    echo "Verifying deployment package..."

    local BINARY=$(find deploy/ -maxdepth 1 -type f -executable | grep -v "\.sh$" | head -n1)

    if [ -z "$BINARY" ]; then
        echo "ERROR: No binary found"
        return 1
    fi

    echo "Checking binary: $(basename $BINARY)"
    echo ""

    # Check glibc version
    echo "GLIBC version requirements:"
    objdump -T "$BINARY" | grep GLIBC | sed 's/.*GLIBC_/GLIBC_/' | sort -u | head -5

    echo ""
    echo "Missing dependencies (should be none if in deploy/lib):"
    LD_LIBRARY_PATH=deploy/lib ldd "$BINARY" | grep "not found" || echo "[OK] All dependencies satisfied"

    echo ""
    echo "[OK] Package verification complete"
}

# Main execution
main() {
    # Check if --install-deps flag is provided
    if [ "$1" == "--install-deps" ]; then
        install_deps
        echo ""
        echo "Run 'source ~/.bashrc' to load environment variables"
        echo "Then run './gnubuilds.sh' to build"
        exit 0
    fi

    # Verify environment before building
    verify_environment
    setup_libtorch
    build_project
    package_for_rhel
    verify_package
    create_tarball
}

main "$@"
