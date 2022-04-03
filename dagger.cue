package chappaai

import (
	"dagger.io/dagger"
	"universe.dagger.io/docker"
	// operator "chappaai.dev/dagger/operator:build"
)

dagger.#Plan & {
	client: {
		filesystem: {
			"./operator": read: {
				path:     "./operator"
				contents: dagger.#FS
			}
			"./web": read: {
				path:     "./web"
				contents: dagger.#FS
			}
		}
	}

	actions: {
		build: docker.#Dockerfile & {
			source: client.filesystem."./operator".read.contents
		}
	}
}
