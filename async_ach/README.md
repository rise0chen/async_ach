# Async Ach

## Features

- `const`: static friendly
- `no_std`: Can run in embedded devices
- `no_alloc`: Needn't dynamic memory allocation
- Lock Free
- Wait Free: `try_send`/`try_recv` is Wait Free
- Async: `send`/`recv` is async

## Usage

### Waker

An array of `core::task::Waker`.

### Notify

Wait for wake.

### Cell

It is similar to RwLock.

### Watch

wake on changed.

### Spsc

bounded SPSC queue.

### Ring

bounded ring buffer.

### Mpmc

bounded MPMC queue.

### Pubsub

broadcast channel.
