package chappaai

import (
	"dagger.io/dagger"
	operator "chappaai.dev/dagger/operator:build"
)

dagger.#Plan & {
	client: {
		filesystem: {
			"./operator": read: {
				contents: dagger.#FS
			}
			"./web": read: {
				contents: dagger.#FS
			}
		}
	}

	actions: {
		build: operator.#Build & {
			_source: client.filesystem."./operator".read.contents
		}
	}
}
