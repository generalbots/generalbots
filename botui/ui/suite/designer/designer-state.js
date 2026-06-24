const state = {
  nodes: new Map(),
  connections: [],
  selectedNode: null,
  selectedConnection: null,
  isDragging: false,
  isConnecting: false,
  connectionStart: null,
  zoom: 1,
  pan: { x: 0, y: 0 },
  history: [],
  historyIndex: -1,
  clipboard: null,
  nextNodeId: 1,
  driveSource: null
};

const nodeTemplates = {
  'TALK': {
    fields: [
      { name: 'message', label: 'Message', type: 'textarea', default: 'Hello!' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'HEAR': {
    fields: [
      { name: 'variable', label: 'Variable', type: 'text', default: 'response' },
      { name: 'type', label: 'Type', type: 'select', options: ['STRING', 'NUMBER', 'DATE', 'EMAIL', 'PHONE'], default: 'STRING' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'SET': {
    fields: [
      { name: 'variable', label: 'Variable', type: 'text', default: 'value' },
      { name: 'expression', label: 'Expression', type: 'text', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'IF': {
    fields: [
      { name: 'condition', label: 'Condition', type: 'text', default: 'value = 1' }
    ],
    hasInput: true,
    hasOutput: false,
    hasOutputTrue: true,
    hasOutputFalse: true
  },
  'FOR': {
    fields: [
      { name: 'variable', label: 'Item Variable', type: 'text', default: 'item' },
      { name: 'collection', label: 'Collection', type: 'text', default: 'items' }
    ],
    hasInput: true,
    hasOutput: true,
    hasLoopOutput: true
  },
  'SWITCH': {
    fields: [
      { name: 'expression', label: 'Expression', type: 'text', default: 'value' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'CALL': {
    fields: [
      { name: 'procedure', label: 'Procedure', type: 'text', default: '' },
      { name: 'arguments', label: 'Arguments', type: 'text', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'SEND MAIL': {
    fields: [
      { name: 'to', label: 'To', type: 'text', default: '' },
      { name: 'subject', label: 'Subject', type: 'text', default: '' },
      { name: 'body', label: 'Body', type: 'textarea', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'GET': {
    fields: [
      { name: 'url', label: 'URL', type: 'text', default: '' },
      { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'POST': {
    fields: [
      { name: 'url', label: 'URL', type: 'text', default: '' },
      { name: 'body', label: 'Body', type: 'textarea', default: '' },
      { name: 'variable', label: 'Result Variable', type: 'text', default: 'result' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'SAVE': {
    fields: [
      { name: 'filename', label: 'Filename', type: 'text', default: 'data.csv' },
      { name: 'data', label: 'Data', type: 'text', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'WAIT': {
    fields: [
      { name: 'duration', label: 'Duration (seconds)', type: 'text', default: '5' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'SET BOT MEMORY': {
    fields: [
      { name: 'key', label: 'Key', type: 'text', default: '' },
      { name: 'value', label: 'Value', type: 'text', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'GET BOT MEMORY': {
    fields: [
      { name: 'key', label: 'Key', type: 'text', default: '' },
      { name: 'variable', label: 'Variable', type: 'text', default: 'value' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'SET USER MEMORY': {
    fields: [
      { name: 'key', label: 'Key', type: 'text', default: '' },
      { name: 'value', label: 'Value', type: 'text', default: '' }
    ],
    hasInput: true,
    hasOutput: true
  },
  'GET USER MEMORY': {
    fields: [
      { name: 'key', label: 'Key', type: 'text', default: '' },
      { name: 'variable', label: 'Variable', type: 'text', default: 'value' }
    ],
    hasInput: true,
    hasOutput: true
  }
};
