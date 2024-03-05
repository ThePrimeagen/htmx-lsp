This attribute ensures that the loading state changes are applied only after 200ms if the request is not finished. The default delay can be modified through the attribute value and expressed in milliseconds:

Example:
<button type="submit" data-loading-disable data-loading-delay="1000">Submit</button>

Note:
You can place the `data-loading-delay` attribute directly on the element you want to disable, or in any parent element.
