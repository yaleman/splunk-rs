# splunk

A start on implementing a rust crate for Splunk-related things.

## TODO

- Most of it
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
  - [ ] searchlog (<http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Fsearch.log>)
  - [ ] set_priority (0-10)
  - [ ] summary (GET search/jobs/{search_id}/summary <http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Fsummary>)
  - [ ] timeline GET search/jobs/{search_id}/timeline <http://docs.splunk.com/Documentation/Splunk/latest/RESTAPI/RESTsearch#GET_search.2Fjobs.2F.7Bsearch_id.7D.2Ftimeline>`
  - [ ] touch (set ttl)
- SearchJob Results - maybe its own thing, maybe an Iterator?
- HEC Transfers
