.PHONY: build clean serve

build: 
	wasm-pack build --release --no-typescript --target web
	mkdir -p public/mod
	cp pkg/* public/mod

	rm -rf pkg	
	wasm-pack build --release --no-typescript --target no-modules
	mkdir -p public/nomod
	cp pkg/* public/nomod
	
	cp index.html public
	cp index.js public
	cp sw*.js public
	rm -rf pkg

clean:
	rm -rf public

serve: build
	python3 -m http.server -d public

all: build
