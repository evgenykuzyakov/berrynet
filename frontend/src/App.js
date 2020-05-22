import React from 'react';
import './App.css';
import { GameWorld } from './GameWorld/GameWorld.js'

class App extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      connected: false,
      userId: null,
      users: {},
    }

    this.sse = null;
    this.apiBaseUri = "http://" + document.location.hostname + ":3030";
  }

  componentDidMount() {
    this.sse = new EventSource(this.apiBaseUri + "/sse");
    this.sse.addEventListener('open', () => {
      this.setState({
        connected: true,
      })
    });
    this.sse.addEventListener('user', (msg) => {
      this.setState({
        userId: msg.data,
      }, () => {
        this.sendLocation({
          x: Math.round(Math.random() * 10000),
          y: Math.round(Math.random() * 10000),
        })
      });
    });
    this.sse.addEventListener("location", (msg) => {
      let userLocation;
      try {
        userLocation = JSON.parse(msg.data);
      } catch (e) {
        console.error(e);
      }
      this.updateUserLocation(userLocation);
    });
    this.sse.addEventListener("kickout", (msg) => {
      let userIds;
      try {
        userIds = JSON.parse(msg.data);
      } catch (e) {
        console.error(e);
      }
      this.removeUsers(userIds);
    });
  }

  updateUserLocation(userLocation) {
    const users = JSON.parse(JSON.stringify(this.state.users));
    const userId = userLocation.user_id;
    const location = userLocation.location;
    const user = users[userId] || {
      userId,
    };
    user.location = location;
    users[user.userId] = user;
    this.setState({
      users,
    })
  }

  removeUsers(userIds) {
    const users = JSON.parse(JSON.stringify(this.state.users));
    userIds.forEach(userId => {
      delete users[userIds];
    });
    this.setState({
      users,
    })
  }

  async sendLocation(location) {
    let _Zres = await postData(this.apiBaseUri + "/move/" + this.state.userId, location);
  }


  render() {
    return (
      <div>
        <div>
          <h2>Berry Net</h2>
        </div>
        <div>
          {this.state.connected ? "Connected" : "Connecting..."}
        </div>
        <div>
          {this.state.userId !== null ? ("Hello user #" + this.state.userId) : "Awaiting userId"}
        </div>
        <GameWorld userId={this.state.userId} users={this.state.users} onClick={(x, y) => this.sendLocation({x, y})}></GameWorld>
      </div>
    );
  }
}

const postData = async (url = '', data = {}) => {
  // Default options are marked with *
  const response = await fetch(url, {
    method: 'POST', // *GET, POST, PUT, DELETE, etc.
    mode: 'no-cors', // no-cors, *cors, same-origin
    cache: 'no-cache', // *default, no-cache, reload, force-cache, only-if-cached
    // credentials: 'same-origin', // include, *same-origin, omit
    headers: {
      'Content-Type': 'application/json'
      // 'Content-Type': 'application/x-www-form-urlencoded',
    },
    redirect: 'follow', // manual, *follow, error
    referrerPolicy: 'no-referrer', // no-referrer, *no-referrer-when-downgrade, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin, unsafe-url
    body: JSON.stringify(data) // body data type must match "Content-Type" header
  });

  return response; // parses JSON response into native JavaScript objects
}

export default App;
