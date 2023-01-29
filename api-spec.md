The server will give a 400 error for any requests that are formatted wrong.

If a request requires authorization, the server will return a `403` error if the authorization is wrong.

# Accounts

## Creating an account

Post the server a JSON object formatted as below to the route `/api/create-account`:

```
  {
    username: string,
    name: string,
    address: string,
    location: [number, number], // [latitude, longitude]
    userType: "Volunteer" | "Senior",
    password: string,
  }
```

The server will give a `409` error if the username already exists, otherwise it will give an authorization string.

## Logging in

Post the server a JSON object formatted as below to the route `/api/login`:

```
  {
    username: string,
    password: string,
  }
```

The server will give a `409` error if the username doesn't exist or a `403` error if the password is incorrect. Otherwise it will give an authorization string.

## Getting user data

To get the user's data, send a JSON object formatted as below to `/api/user-data`

```
  {
    authorization: Authorization string
  }
```

The server will return a JSON object formatted as below:

```
  {
    username: string,
    name: string,
    address: string,
    location: [number, number], // [lat, long]
    user_type: { Volunteer: string[] } | { Senior: string | null }, // Volunteer contains a list of every request ID they've accepted, Senior contains the request ID of the request they've made
  }
```

## Authorization string

This is a string formatted as:

```
  {
    username: string,
    expirationTime: number,
    nonce: number[],
    mac: number[],
  }
```

The string does not need to be parsed, and must be given back to the server verbatim whenever authorization is needed.

# `User`

`User` is a json object formatted as below:

```
  {
    username: string,
    name: string,
  }
```

It represents the data that can be made public for a user.

# Help requests

All of these endpoints will return a `405` error if the user isn't a `Senior`

## Making a help request

To make a help request, post a JSON object formatted as below to the route `/api/request-help`

```
  {
    picture: Type TBD, // Let me know when you figure out how to do picture uploads and what format the data comes in
    notes: string,
    authorization: Authorization string,
  }
```

The server will respond with a `409` error if the user already created a help request

## Getting help requests

To get a user's help request, post a JSON object formatted as below to `/api/help-requests`

```
  {
    authorization: Authorization string
  }
```

The server will respond with the JSON object below if there is one, or a `409` error if not. If a help request is completed, the server will delete it and this will return a `409` error.

```
  {
    picture: Type TBD,
    notes: string,
    creationTime: number, // Milliseconds since UNIX epoch
    state: "Pending" | { AcceptedBy: User } | { MarkedCompletedBy: User } // The users in the AcceptedBy and MarkedCompletedBy variants are guaranteed to be the same.
  }
```

## Deleting help requests

To delete a help request, post a JSON object formatted as below to `/api/delete-help-request`

```
  {
    authorization: Authorization string
  }
```

# Volunteering

All of these endpoints will return a `405` error if the user isn't a `Volunteer`

## Requesting work

To request volunteer work, post a JSON object formatted as below to the route `/api/request-work`

```
  {
    authorization: Authorization string,
  }
```

The server will respond with:

```
  [
    string, // The ID of the request
    ...
  ]
```

The array may be of any length or empty. The array will be sorted by distance, lowest to highest.

## Getting a request by ID

To get a volunteer request by id, post a JSON object as below to `/api/get-request`

```
  {
    id: string,
    authorization: Authorization string,
  }
```

The server will respond with the JSON object as below if the id exists, otherwise it will give a `409` error.

```
  {
    user: User,
    picture: Type TBD,
    notes: string,
    dist: number, // Units TBD,
    address: string,
  }
```

## Accepting a request

To accept a request, post a JSON object as below to `/api/accept-request`

```
  {
    id: string,
    authorization: Authorization string,
  }
```

## Getting accepted requests

To get all accepted requests, post a JSON object as below to `/api/accepted-requests`

```
  {
    authorization: Authorization string
  }
```

## Marking a request as completed

To mark a request as completed, post a JSON object as below to `/api/mark-request-completed`

```
  {
    id: string,
    authorization: Authorization string,
  }
```

The server will respond with a `409` error if the id doesn't exist or wasn't previously accepted by the user