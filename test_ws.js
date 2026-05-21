const WebSocket = require('ws');

const ws = new WebSocket('ws://localhost:5858/ws/salesianos');

ws.on('open', function open() {
  console.log('Connected to salesianos bot!');
  
  // Send "ramais e cartas" message
  const msg = {
    message_type: 1, // USER message
    content: 'Quais os ramais disponíveis no pdf de ramais?',
    timestamp: new Date().toISOString()
  };
  
  ws.send(JSON.stringify(msg));
  console.log('Sent question about ramais...');
});

ws.on('message', function incoming(data) {
  const response = JSON.parse(data);
  console.log('Bot Response:', response.content || response);
  if (response.message_type === 2) {
    // BOT_RESPONSE
    console.log("Got final response. Closing.");
    setTimeout(() => { ws.close(); process.exit(0); }, 1000);
  }
});

ws.on('error', function error(err) {
  console.error('WebSocket error:', err);
});

setTimeout(() => {
  console.log('Timeout waiting for response');
  ws.close();
  process.exit(1);
}, 15000);
