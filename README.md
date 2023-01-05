# Chappaai

Chappaai is an OAuth management layer for Kubernetes.

It allows you, through CRDs, to describe OAuth APIs you wish to be able to integrate with.

Then, through the web service, you can perform the OAuth dance and a token will be stored, as a secret, within the cluster; available for your workloads.

## Installation

```shell
kubectl apply -k ./deploy
```
