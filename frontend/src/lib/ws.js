import { get } from 'svelte/store';
import { teams, pendingPlan, connected, authToken, currentView } from './stores.js';
import { api } from './api.js';

let ws = null;
let reconnectDelay = 1000;
const MAX_DELAY = 30000;

export function connectWs() {
    const token = get(authToken);
    if (!token) return;
    const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${location.host}/ws?token=${token}`);

    ws.onopen = () => {
        connected.set(true);
        reconnectDelay = 1000;
        syncTeams();
    };
    ws.onmessage = (event) => {
        try {
            handleEvent(JSON.parse(event.data));
        } catch (e) {
            console.warn('[ws] failed to parse message', e);
        }
    };
    ws.onclose = () => {
        connected.set(false);
        setTimeout(() => connectWs(), reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, MAX_DELAY);
    };
    ws.onerror = (err) => {
        console.warn('[ws] error', err);
    };
}

async function syncTeams() {
    try {
        const data = await api.listTeams();
        const map = {};
        for (const team of (data.teams || [])) map[team.id] = team;
        teams.set(map);
    } catch (e) {
        console.warn('[ws] syncTeams failed', e);
    }
}

function handleEvent(msg) {
    if (msg.v !== 1) return;
    switch (msg.type) {
        case 'orchestrator_plan':
            pendingPlan.set(msg);
            currentView.set('plan');
            break;
        case 'team_created':
            teams.update(t => ({ ...t, [msg.team.id]: msg.team }));
            break;
        case 'team_output':
            teams.update(t => {
                if (t[msg.team_id]) {
                    t[msg.team_id] = {
                        ...t[msg.team_id],
                        output: [...(t[msg.team_id].output || []), msg.line]
                    };
                }
                return { ...t };
            });
            break;
        case 'team_done':
            teams.update(t => {
                if (t[msg.team_id]) {
                    t[msg.team_id] = { ...t[msg.team_id], status: 'Done', summary: msg.summary };
                }
                return { ...t };
            });
            break;
        case 'team_error':
            teams.update(t => {
                if (t[msg.team_id]) {
                    t[msg.team_id] = { ...t[msg.team_id], status: 'Failed', error: msg.error };
                }
                return { ...t };
            });
            break;
    }
}

export function disconnectWs() {
    if (ws) {
        ws.onclose = null; // prevent auto-reconnect
        ws.close();
        ws = null;
    }
    connected.set(false);
}
