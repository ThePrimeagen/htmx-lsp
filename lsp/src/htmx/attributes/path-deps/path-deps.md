Set the path dependency of the current div. 
Use together with hx-trigger="path-deps".

Usage:

<div hx-get="/example"
     hx-trigger="path-deps"
     path-deps="/foo/bar">
  ...
</div>

This div will fire a GET request to /example when any other element issues a mutating request (that is, a non-GET request like a POST) to /foo/bar or any sub-paths of that path.

You can use a * to match any path component:

<div hx-get="/example"
     hx-trigger="path-deps"
     path-deps="/contacts/*">
    ...
</div>

