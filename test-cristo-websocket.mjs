import WebSocket from 'ws';

const ws = new WebSocket('ws://localhost:8080/ws?bot_name=cristo&user_id=test-user&session_id=test-session');

let messageCount = 0;

ws.on('open', () => {
  console.log('Connected to WebSocket');
});

ws.on('message', (data) => {
  messageCount++;
  const msg = JSON.parse(data.toString());
  console.log(`MSG ${messageCount}: type=${msg.type}, content=${(msg.content || '(no content)').substring(0, 200)}`);
  
  // After connected + start.bas response, send a user message
  if (messageCount === 2) {
    console.log('\n--- Sending: "Quero agendar um batizado" ---');
    ws.send(JSON.stringify({
      text: 'Quero agendar um batizado',
      message_type: 1
    }));
  }
  
  // After batizado tool response, test child info
  if (messageCount === 4) {
    console.log('\n--- Sending: "Maria da Silva" (child name) ---');
    ws.send(JSON.stringify({
      text: 'Maria da Silva',
      message_type: 1
    }));
  }
});

ws.on('error', (err) => {
  console.error('WebSocket error:', err.message);
});

ws.on('close', () => {
  console.log('WebSocket closed');
});

setTimeout(() => {
  console.log('\nTest complete, keeping connection open for more messages...');
}, 30000);
