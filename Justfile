release:
    cd web
    VERSION=$(npm version patch)
    git add package.json package-lock.json
    cd ..

    cd operator
    cargo bump ${VERSION}
    git add Cargo.{lock,toml}
    cd ..

    git commit -m "release: ${VERSION}"
    git tag ${VERSION}
