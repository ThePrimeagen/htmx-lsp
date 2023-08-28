This extension adds the X-Requested-With header to requests with the value “XMLHttpRequest”.
This header is commonly used by javascript frameworks to differentiate ajax requests from normal http requests.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/ajax-header.js"></script>

Usage
<body hx-ext="ajax-header">
    ...
</body>


[HTMX Reference](https://htmx.org/extensions/ajax-header/)
