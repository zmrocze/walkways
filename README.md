# Walkways

## Overview

The repository contains code for controlling walkways (a type of moving staircase, made of components called platforms) and simulating walkways in software.

There's 2 components: 
  - centre (or main server): monitors platforms and issues Start/Stop signals to platforms
  - platform: controls its part of the track by setting its parameters like speed and acceleration

### Centre/Platform communication

The platforms communicate with the centre via gRPC interface. Both platforms and centre act both as servers and clients. The protocol description is found in [proto](./core/src/proto/) directory.

#### Initialization

1. Both centre and platforms start their gRPC servers. 
2. Platforms call centre's ImReady endpoint.
3. Centre pings ready platforms with the Ping endpoint.
4. Once server has verified all platforms are responsive, it issues Start signal to platforms.

From now on platforms are mostly on their own. They control platform parameters independently and the communication with the centre is just to share information. Platforms alert server about situations and server may query platforms. If a platform reports a critical alert to the server it should issue the Stop signal (though in a critical situation it may not even reach platforms).

### Platform control system

Outside of serving the communication with centre, a platform controls its own movement.
It runs a loop: 

 - load parameters (like our speed and distance to neighbours)
 - calculate controlled parameters (like acceleration)
 - check if we are in a dangerours situation, respond with alert to the centre
 - set the controlled parameters

This is just a loop, but it could be made into an event loop instead, so that the software responds to changing parameters in async fashion. This part is common between the software running in the simulation and on hardware, what changes are the implementations for how to load/set parameters. 

#### Calculating speed

Morally we always aim for a desired speed for the given platform. This may involve setting the acceleration instead, but the goal is to calculate the desired speed.

The desired speed for a platform is a result of the constraints:
1. we want to move as fast as possible, no faster than max allowed speed
2. we need to be able to slow down in time for the track entry points
3. we need to stay in the desired distance range to neighbouring platforms

The constraint 1. is easy to account for. We encode the track as a list of sections - either slow-down sections or go-as-fast-as-possible sections.

Constraints 2. and 3. are somewhat simmilar. In both cases there's a point some known distance in front of us,
such that we should have a known speed when we'll reach the point. In 2. the point is the track entry point and the speed is the desired speed for passengers to exit easily. In 3. it's a bit different, because the platform in front is moving itself. Conservatively, to avoid hitting the front platform, we should have at most the speed of the front platform, when we'll reach the place where the front platform is right now.

We aim at the speed following from points 1. and 2. (that is maximal speed allowing us to slow down in time).
Additionaly constraint 3. may need us to go slower, not to hit the front platform. So to the control system, there's always 'best' speed to go with. The bounds on allowed speed ranges should be part of the independent monitoring system.

> Q: Should we consider the back platform the same, that is possibly speed up, to make room for the back platform, eventhough we don't control it's own speed?

Currently the controller simply sets the acceleration to maximal possible slowing or speeding up platform to reach the desired speed. Instead, for more stable behaviour, a controller should be used, like this one: [pid](https://docs.rs/pid/latest/pid/).
