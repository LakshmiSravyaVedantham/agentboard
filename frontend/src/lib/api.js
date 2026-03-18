import { get } from 'svelte/store';
import { authToken } from './stores.js';

async function request(method, path, body = null) {
    const token = get(authToken);
    const opts = { method, headers: { 'Content-Type': 'application/json' } };
    if (token) opts.headers['Authorization'] = `Bearer ${token}`;
    if (body) opts.body = JSON.stringify(body);
    const resp = await fetch(path, opts);
    if (!resp.ok) {
        const text = await resp.text().catch(() => '');
        throw new Error(`${method} ${path} → ${resp.status}: ${text}`);
    }
    return resp.json();
}

export const api = {
    pair: (code) => request('POST', '/api/auth/pair', { code }),
    health: () => request('GET', '/api/health'),
    submitTasks: (input) => request('POST', '/api/tasks', { input }),
    approvePlan: (planId) => request('POST', '/api/plan/approve', { plan_id: planId }),
    editPlan: (planId, teams) => request('POST', '/api/plan/edit', { plan_id: planId, teams }),
    listTeams: () => request('GET', '/api/teams'),
    getTeam: (id) => request('GET', `/api/teams/${id}`),
    sendMessage: (id, message) => request('POST', `/api/teams/${id}/message`, { message }),
    killTeam: (id) => request('DELETE', `/api/teams/${id}`),
    summary: () => request('GET', '/api/summary'),
};
