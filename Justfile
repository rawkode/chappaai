release:
    #!/usr/bin/env sh
    set -x
    cd web
    VERSION=$(npm version patch)
    git add package.json package-lock.json
    cd ..

    cd operator
    # Slice of the v from vx.x.x returned by npm
    cargo bump ${VERSION:1}
    cargo build
    git add Cargo.toml Cargo.lock
    cd ..

    git commit -m "release: ${VERSION:1}"
    git tag ${VERSION:1}
