# `atpage`

Publish your website on an ATProto PDS.

> [!WARNING]
> WIP
>
> Nothing works
>
> The only website that has this thing deployed is https://geesawra.industries, but it's just a demo.

## Requirements

 - [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)


## Build and run

```sh
make
```

The `public` directory content can be placed at the root of your website.
To serve a local copy in on `0.0.0.0:8000` (needs Python 3 installed):

```sh
make serve
```

Not intended as a way of deployment, it's a development tool.

## Limitations

Only works in WebKit and Blink-based browsers, Firefox needs to figure out how to use ES modules in service workers first.
