# syntax=docker/dockerfile:1
ARG BIN_PATH=./bicep-docs

FROM alpine:3.22

COPY ${BIN_PATH} /bicep-docs

ENTRYPOINT ["/bicep-docs"]
