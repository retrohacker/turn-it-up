# TURN It Up

Automated NAT traversal for everyday users

# Project Thesis

Most users on the internet expect software to either run:

- in a browser,
- or on a phone.

This is a problem for peer-to-peer software. This project is an attempt to solve
this problem.

Imagine a Peer-to-Peer social network. You connect directly to your friends
devices to share and discover content (i.e.
[scuttlebutt](https://ssbc.github.io/ssb-db/) meets [IPFS](https://ipfs.tech/)).

For most "social graphs" a majority of a user's peer group (their friends and
family) will either be using a browser or using a cellphone. These devices can
not establish direct peer-to-peer connections on their own on the current
internet (for why, read this [blog post](https://ipfs.tech/)).

What we need to connect these devices is a long-lived network appliance with a
public IP address that can act as a rendezvous point for these devices, helping
them get through NAT, and providing relay services when NAT traversal fails.

For a resilient and truely p2p network, these network appliances should be
deployed to a majority of the network user's homes.

Luckily, most households already have a long lived computer that has a public IP
address: their home router.

This project's goal is to enable non-technical end users to host a peer-to-peer
rendezvous server using their existing router.

This software attempts to "fingerprint" a router and automatically configure it
to act as a p2p relay.

The p2p relay will configure itself to offer peer-to-peer connectivity to the
user's social group (their friends and family).

When deployed as a network device (i.e. using a raspberry pi) and end-user
should be able to plug this appliance in to the ethernet port on their router
and get a working rendezvous point with minimal configuration necessary.

# Status

This software is _alpha_.

It should work for all modern Asus routers (running the Asus WRT software or
Merlin).

# Todo

- [ ] Create a Web UI for communicating status of the appliance and prompting
      for the router's username and password
- [ ] Create a network service for redirecting a user to the local appliance
      during initial setup
- [ ] Ingress a user's "friends list" for white listing peers to provide
      services to
- [ ] Create a systemd configuration for self-starting the service at boot
- [ ] Verify public IP connectivity after self configuration
- [ ] Update DCUtR to start in server mode and announce its public IP address
      instead of local addresses
- [ ] Update router config to pull a static IP address from the dhcp server
- [ ] Deploy to a raspberry pi
- [ ] Auto-update
- [ ] Increase router model coverage
