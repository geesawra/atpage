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
