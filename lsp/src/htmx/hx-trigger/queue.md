queue:<queue option> - determines how events are queued if an event occurs while a request for another event is in flight. Options are:

* first - queue the first event
* last - queue the last event (default)
* all - queue all events (issue a request for each event)
* none - do not queue new events



[HTMX Reference](https://htmx.org/attributes/hx-trigger/)
