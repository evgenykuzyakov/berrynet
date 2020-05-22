import React from 'react';
import './GameWorld.css';

class GameWorld extends React.Component {
  constructor(props) {
    super(props);

    this.world = React.createRef();
  }

  componentDidMount() {
    this.world.current.addEventListener('click', (e) => {
      const rect = this.world.current.getBoundingClientRect();
      let x = Math.round((e.clientX - rect.left) / rect.width * 10000);
      let y = Math.round((e.clientY - rect.top) / rect.height * 10000);
      this.props.onClick(x, y);
    })
  }

  render() {
    // const userId = this.props.userId;
    const users = this.props.users;
    let circles = [];
    if (this.world.current) {
      const rect = this.world.current.getBoundingClientRect();
      circles = Object.values(users).map(user => {
        const localLocation = {
          left: rect.width * (user.location.x / 10000),
          top: rect.height * (user.location.y / 10000),
        }
        return (
          <div key={"user-" + user.userId} className="user" style={localLocation}>User #{user.userId}</div>
        )
      });
    }

    return (
      <div className="game-world" ref={this.world}>
        {circles}
      </div>
    );
  }
}

export { GameWorld };
