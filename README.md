

<h1 align="center">
  <p align="center">Kube-depre</p>
</h1>

<div align="center">
  <a href="hhttps://github.com/anzx/platform-secret-management/actions/workflows/ci.yaml" alt="Build"><img src="https://github.com/maheshrayas/kube-depre/actions/workflows/ci.yaml/badge.svg" /></a>
  <a href="https://codecov.io/gh/maheshrayas/kube-depre" alt="Lint"><img src="https://codecov.io/gh/maheshrayas/kube-depre/branch/main/graph/badge.svg?token=VF6UCCDNXI" /></a>
</div>


## Motivation

Given that kubernetes frequently deprecates apiVersions, we want to check for the resources with deprecated apiVersions in cluster or files or as a part of Continous Integration pipeline (Github Actions) so that we can update the apiVersion in manifest before the cluster is upgraded.

`kube-depre` is a simple CLI tool that allows us to find such deprecated apiVersion in Kubernetes cluster, or in files and as well integrated with github actions to report the Deprecated Apis as a comment on Pull Request.

## Installation
