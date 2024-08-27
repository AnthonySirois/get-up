## GET UP

Get up is a simple (read barebones) tool meant to be used with a standing desk. It sends out a notification every time the timer ends, reminding the user to either sit or stand, depending on their current stance. The duration of the timer alternates between the time spent sitting and the time spent standing.

### Usage

```
get-up fixed <SECONDS SITTING> <SECONDS STANDING>
```

The timer starts as sitting by default. This can be changed by passing the `--standing` flag.