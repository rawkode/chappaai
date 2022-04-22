package chappaai

import (
	"dagger.io/dagger"
	operator "chappaai.dev/dagger/operator:build"
)

dagger.#Plan & {
	platform: "linux/aarch64"
	client: {
		filesystem: {
			"./": read: {
				contents: dagger.#FS
			}
		}
	}

	actions: {
		build: operator.#Build
	}
}
