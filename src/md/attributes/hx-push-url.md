hx-push-url

The hx-push-url attribute allows you to push a URL into the browser location history. This creates a new history entry, allowing navigation with the browser’s back and forward buttons. htmx snapshots the current DOM and saves it into its history cache, and restores from this cache on navigation.

The possible values of this attribute are:

    true, which pushes the fetched URL into history.
    false, which disables pushing the fetched URL if it would otherwise be pushed due to inheritance or hx-boost.
    A URL to be pushed into the location bar. This may be relative or absolute, as per history.pushState().

Here is an example:

<div hx-get="/account" hx-push-url="true">
  Go to My Account
</div>

This will cause htmx to snapshot the current DOM to localStorage and push the URL `/account’ into the browser location bar.

Another example:

<div hx-get="/account" hx-push-url="/account/home">
  Go to My Account
</div>

This will push the URL `/account/home’ into the location history.
Notes

    hx-push-url is inherited and can be placed on a parent element
    The HX-Push-Url response header has similar behavior and can override this attribute.
    The hx-history-elt attribute allows changing which element is saved in the history cache.

[HTMX Reference](https://htmx.org/attributes/hx-push-url/)
