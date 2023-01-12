set fallback := true

_default:
    @echo bbl2csv:
    @just --list --unsorted --list-heading ''
    @echo
    @echo Global:
    @cd .. && just --list --unsorted --list-heading ''

# Profile using cargo-flamegraph
profile *args='':
    cargo flamegraph --deterministic --palette rust
