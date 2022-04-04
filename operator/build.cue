package build

import (
	"dagger.io/dagger/core"
	"universe.dagger.io/docker"
)

#Build: {
	_source: core.#Source & {
		path: "."
	}

	image: docker.#Dockerfile & {
		source: _source.output
	}
}
