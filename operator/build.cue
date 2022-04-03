package build

import (
	"dagger.io/dagger"
	"universe.dagger.io/docker"
)

#Build: {
	_source: dagger.#FS

	image: docker.#Dockerfile & {
		source: _source
	}
}
