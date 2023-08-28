The hx-on attribute allows you to embed scripts inline to respond to events directly on an element; similar to the onevent properties found in HTML, such as onClick.

hx-on improves upon onevent by enabling the handling of any event for enhanced Locality of Behaviour (LoB). This also enables you to handle any htmx event.

There are two forms of this attribute, one in which you specify the event as part of the attribute name after a colon (hx-on:click, for example), and a deprecated form that uses the hx-on attribute directly. The latter should only be used if IE11 support is required.
hx-on:* (recommended)

The event name follows a colon : in the attribute, and the attribute value is the script to be executed:

<div hx-on:click="alert('Clicked!')">Click</div>

All htmx events can be captured, too! Make sure to use the kebab-case event name, because DOM attributes do not preserve casing. For instance, hx-on::beforeRequest will not work: use hx-on::before-request instead.

To make writing these a little easier, you can use the shorthand double-colon hx-on:: for htmx events, and omit the “htmx” part:

<!-- These two are equivalent -->
<button hx-get="/info" hx-on:htmx:before-request="alert('Making a request!')">
    Get Info!
</button>

<button hx-get="/info" hx-on::before-request="alert('Making a request!')">
    Get Info!
</button>

Adding multiple handlers is easy, you just specify additional attributes:

<button hx-get="/info"
        hx-on::before-request="alert('Making a request!')"
        hx-on::after-request="alert('Done making a request!')">
    Get Info!
</button>

Symbols

Like onevent, two symbols are made available to event handler scripts:

    this - The element on which the hx-on attribute is defined
    event - The event that triggered the handler

Notes

    hx-on is not inherited, however due to event bubbling, hx-on attributes on parent elements will typically be triggered by events on child elements
    hx-on:* and hx-on cannot be used together on the same element; if hx-on:* is present, the value of an hx-on attribute on the same element will be ignored. The two forms can be mixed in the same document, however.

[HTMX Reference](https://htmx.org/attributes/hx-on/)
