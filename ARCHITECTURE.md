# A website's architecture under atpage

`baseURL` is the URL at which the website should be uploaded.

Anything that's referenced at `baseURL` will be uploaded to the configured PDS.

## AT URI definition

For the Service Worker to pick up a resolution job, it needs a URL in the following form:

```
http{s}://{current href URL}/at/{AT URI}
```

AT URIs are defined [here](https://atproto.com/specs/at-uri-scheme).

We use the current `href` due to CORS constraints: we just need a way to pipe `fetch()` events to the Service Worker.


## Lexicon (?)

```
PDS ->
  Collections ->
    industries.geesawra.webpages -> // this is where your web pages are stored
      record key                    // this holds a page's content
```

Record key are defined [here](https://atproto.com/specs/record-key).


## Architecture v2

One record = one `Page` = one text resource.

HTML is a record, while CSS, JS and images are stored as `blob` objects.

Each record holds a list of `blob` associated with it (`blob` refs need to be referenced by a `Record` otherwise they get discarded, and on top of that this way we have an easy path to deletion).

```rust
struct Page {
  content: String,
  embeds: Vec<Blob>
}
```
