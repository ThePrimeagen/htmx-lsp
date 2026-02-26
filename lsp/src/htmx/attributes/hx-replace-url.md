The hx-replace-url attribute allows you to replace the current url of the browser location history.

The possible values of this attribute are:

    true, which replaces the fetched URL in the browser navigation bar.
    false, which disables replacing the fetched URL if it would otherwise be replaced due to inheritance.
    A URL to be replaced into the location bar. This may be relative or absolute, as per history.replaceState().

Here is an example:

```html
<div hx-get="/account" hx-replace-url="true">
  Go to My Account
</div>
```

This will cause htmx to snapshot the current DOM to localStorage and replace the URL `/account’ in the browser location bar.

Another example:

```html
<div hx-get="/account" hx-replace-url="/account/home">
  Go to My Account
</div>
```

This will replace the URL `/account/home’ in the browser location bar.
Notes

    hx-replace-url is inherited and can be placed on a parent element
    The HX-Replace-Url response header has similar behavior and can override this attribute.
    The hx-history-elt attribute allows changing which element is saved in the history cache.
    The hx-push-url attribute is a similar and more commonly used attribute, which creates a new history entry rather than replacing the current one.


[HTMX Reference](https://htmx.org/attributes/hx-replace-url/)
