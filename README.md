# splunk

A start on implementing a Rust crate for Splunk-related things.

Check out the tests and examples in the source repository for some
implementation hints.

This is async, no blocking thanks!

## TODO

- Most of it!
- HEC Transfers
  - [x] send_event sends a single event
  - [x] if you want to batch up things, you can use send_events and/or
        HecClient.enqueue() / HecClient.flush()
- REST API Auth
  - [x] Basic Authentication to the REST API
  - [ ] Token Authentication to the REST API
  - [ ] Cookie-based Authentication to the REST API
- REST API SearchJob
  - [ ] create `<http://dev.splunk.com/view/SP-CAAAEE5#searchjobparams>`
    - [ ] disable preview
    - [ ] enable preview
    - [ ] events handle
  - [ ] export
  - [ ] oneshot
  - [ ] cancel
  - [ ] finalize job
  - [ ] is_done
  - [ ] is_ready
  - [ ] name getter (search ID)
  - [ ] pause / unpause
  - [ ] searchlog
        (<http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Fsearch.log>)
  - [ ] set_priority (0-10)
  - [ ] summary (GET search/jobs/{search_id}/summary
        <http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Fsummary>)
  - [ ] timeline GET search/jobs/{search_id}/timeline
        <http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Ftimeline>`
  - [ ] touch the job (set ttl)
- SearchJob Results - maybe its own thing, maybe an Iterator?

## Thanks

In no particular order:

- [reqwest](https://crates.io/crates/reqwest)
- [serde_json](https://crates.io/crates/serde_json)
- [serde](https://crates.io/crates/serde)
- [clap](https://crates.io/crates/clap)
- [tokio](https://crates.io/crates/tokio)
