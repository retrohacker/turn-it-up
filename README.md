# TURN It Up

Proof of concept.

In order to provide browser-to-browser connectivity for P2P experiences where
end users expect application experiences to run (namely browser tabs and mobile
devices) we need to solve point-to-point connections for these devices.

But that means solving WebRTC and carrier-grade NAT traversal!

In order for end users to bring their own NAT traversal solutions they need a
long running computer with a public IP address.

Luckily, most households that are connected to the internet already have one of
these! A home router!

This repository contains a proof-of-concept for automatically detecting and
configuring a RT-AC5300.

Instead of installing software on the router itself, this proof-of-concept
relies on the exposed HTTP API of the router to enable packet forwarding to one
or more services running on the Raspberry Pi.

The idea is to deploy a self-configuring device (i.e. a Raspberry Pi) that
guides a user through a white-glove experience to "upgrade" their home internet
connection with a peer-to-peer rendezvous service exposing NAT traversal
strategies for their devices to use.
