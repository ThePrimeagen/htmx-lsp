The include-vals extension allows you to programmatically include values in a request with a include-vals attribute. The value of this attribute is one or more name/value pairs, which will be evaluated as the fields in a javascript object literal.

Install
<script src="https://unpkg.com/htmx.org/dist/ext/include-vals.js"></script>

Usage
<div hx-ext="include-vals">
    <div hx-get="/test" include-vals="included:true, computed: computeValue()">
      Will Include Additional Values
    </div>
</div>

[HTMX Reference](https://htmx.org/extensions/include-vals/)
