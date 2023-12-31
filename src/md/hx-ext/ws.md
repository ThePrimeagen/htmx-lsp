The WebSockets extension enables easy, bi-directional communication with Web Sockets servers directly from HTML. This replaces the experimental hx-ws attribute built into previous versions of htmx. For help migrating from older versions, see the Migrating guide at the bottom of this page.

Use the following attributes to configure how WebSockets behave:

* ws-connect="<url>" or ws-connect="<prefix>:<url>" - A URL to establish an WebSocket connection against.
* Prefixes ws or wss can optionally be specified. If not specified, HTMX defaults to add the location’s scheme-type, host and port to have browsers send cookies via websockets.
* ws-send - Sends a message to the nearest websocket based on the trigger value for the element (either the natural event or the event specified by [hx-trigger])

Install
<script src="https://unpkg.com/htmx.org/dist/ext/ws.js"></script>

Usage
<div hx-ext="ws" ws-connect="/chatroom">
    <div id="notifications"></div>
    <div id="chat_room">
        ...
    </div>
    <form id="form" ws-send>
        <input name="chat_message">
    </form>
</div>

Configuration

WebSockets extension support two configuration options:

* createWebSocket - a factory function that can be used to create a custom WebSocket instances. Must be a function, returning WebSocket object
* wsBinaryType - a string value, that defines socket’s binaryType property. Default value is blob

Receiving Messages from a WebSocket

The example above establishes a WebSocket to the /chatroom end point. Content that is sent down from the websocket will be parsed as HTML and swapped in by the id property, using the same logic as Out of Band Swaps.

As such, if you want to change the swapping method (e.g., append content at the end of an element or delegate swapping to an extension), you need to specify that in the message body, sent by the server.

<!-- will be interpreted as hx-swap-oob="true" by default -->
<form id="form">
    ...
</form>
<!-- will be appended to #notifications div -->
<div id="notifications" hx-swap-oob="beforeend">
    New message received
</div>
<!-- will be swapped using an extension -->
<div id="chat_room" hx-swap-oob="morphdom">
    ....
</div>

Sending Messages to a WebSocket

In the example above, the form uses the ws-send attribute to indicate that when it is submitted, the form values should be serialized as JSON and send to the nearest enclosing WebSocket, in this case the /chatroom endpoint.

The serialized values will include a field, HEADERS, that includes the headers normally submitted with an htmx request.

Automatic Reconnection

If the WebSocket is closed unexpectedly, due to Abnormal Closure, Service Restart or Try Again Later, this extension will attempt to reconnect until the connection is reestablished.

By default, the extension uses a full-jitter exponential-backoff algorithm that chooses a randomized retry delay that grows exponentially over time. You can use a different algorithm by writing it into htmx.config.wsReconnectDelay. This function takes a single parameter, the number of retries, and returns the time (in milliseconds) to wait before trying again.

// example reconnect delay that you shouldn't use because
// it's not as good as the algorithm that's already in place
htmx.config.wsReconnectDelay = function (retryCount) {
    return retryCount * 1000 // return value in milliseconds
}

The extension also implements a simple queuing mechanism that keeps messages in memory when the socket is not in OPEN state and sends them once the connection is restored.

[HTMX Reference](https://htmx.org/extensions/web-sockets/)
