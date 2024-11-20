.PHONY: build clean serve

build: 
	wasm-pack build --release --no-typescript --target web
	mkdir -p public/pkg
	cp pkg/* public/pkg
	cp index.html public
	cp sw.js public
	rm -rf pkg

clean:
	rm -rf public

serve: build
	python3 -m http.server -d public

all: build
