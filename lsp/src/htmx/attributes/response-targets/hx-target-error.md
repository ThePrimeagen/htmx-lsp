when a 5xx or 4xx error is returned. Target a different element.

<div hx-ext="response-targets">
    <div id="response-div"></div>
    <button hx-post="/register"
            hx-target="#response-div"
            hx-target-error="#any-errors">
        Register!
    </button>
    <div id="any-errors"></div>
</div>

2xx codes will be handled as normal. However, when the response code is 5xx or 4xx, the response from /register will replace the contents of the div with the id any-errors.
