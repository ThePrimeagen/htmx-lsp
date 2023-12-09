The hx-boost attribute allows you to “boost” normal anchors and form tags to use AJAX instead. This has the nice fallback that, if the user does not have javascript enabled, the site will continue to work.

Notes
hx-boost is inherited and can be placed on a parent element
Only links that are to the same domain and that are not local anchors will be boosted
All requests are done via AJAX, so keep that in mind when doing things like redirects
To find out if the request results from a boosted anchor or form, look for HX-Boosted in the request header

[HTMX Reference](https://htmx.org/attributes/hx-boost/)
