# broxi

Broxi is a Batching & Buffering HTTP Proxy.  A request to `broxi` specfiies one or more outgoing requests to be made, and returns after all requests have been attempted (or a timeout elapsed).  `broxi` makes requests concurrently, up to some concurrency limit.  The limit for each host is determined dynamically based on observed 429 responses & TCP rejections.  In this way, `broxi` attempts to maximize throughput within the rate limit, while providing reasonable latency & backpressure to its clients.

Outgoing requests are made in order of arrival, and may queue for some time waiting for an available connection.  Clients should specify a timeout which balances how long they are willing to wait with the cost of retrying requests which time out.
Requests will not be made after the timeout.  The response from `broxi` to its client indicates which requests completed, were canceled without attempt, or were in flight or otherwise indeterminate.  Clients should typically retry requests which were never attempted after some delay.  Clients may retry in-flight requests (at-least-once delivery) or abandon them (at-most-once delivery).
