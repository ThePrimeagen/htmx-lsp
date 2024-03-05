 The value of this attribute is one or more name/value pairs, which will be evaluated as the fields in a javascript object literal.

Usage:
<div hx-ext="include-vals">
    <div hx-get="/test" include-vals="included:true, computed: computeValue()">
      Will Include Additional Values
    </div>
</div>
