import { ActionInfo, MatchItem, UploadStatus } from 'src/types';

/** Shape of `status` in matches stored before it became an `UploadStatus`. */
type LegacyStatus = { response?: string; error?: string };

/** Raw command fields older versions stored per action; nothing displays them. */
type LegacyAction = ActionInfo & {
    data?: unknown;
    command?: { custom?: unknown; raw_context?: unknown; receivers?: unknown } | null;
};

/** A match as read from store.json, which may predate `UploadStatus` or carry raw action bytes. */
export type StoredMatch = Omit<MatchItem, 'status' | 'actions'> & {
    status: UploadStatus | LegacyStatus;
    actions: LegacyAction[];
};

/** Upgrades an older store.json entry; returns the same object when nothing needed changing. */
export function migrateStoredMatch(match: StoredMatch): MatchItem {
    const status = migrateStatus(match.status);
    const actions = stripRawActionFields(match.actions);

    if (status === match.status && actions === match.actions) {
        return match as MatchItem;
    }

    return { ...match, status, actions };
}

/** Older store.json entries hold the server's `{response}` / `{error}` body. */
function migrateStatus(status: UploadStatus | LegacyStatus): UploadStatus {
    if ('kind' in status) {
        return status;
    }

    return status.response !== undefined
        ? { kind: 'accepted', message: status.response }
        : { kind: 'rejected', message: status.error ?? '' };
}

function hasRawFields(action: LegacyAction): boolean {
    const command = action.command;
    return (
        'data' in action ||
        (!!command && ('custom' in command || 'raw_context' in command || 'receivers' in command))
    );
}

function stripRawActionFields(actions: LegacyAction[]): ActionInfo[] {
    if (!actions?.some(hasRawFields)) {
        return actions;
    }

    return actions.map(({ data, command, ...action }) => {
        if (!command) {
            return { ...action, command };
        }
        const { custom, raw_context, receivers, ...kept } = command;
        return { ...action, command: kept };
    });
}

export function uploadStatusText(status: UploadStatus): string {
    switch (status.kind) {
        case 'accepted':
        case 'rejected':
            return status.message;
        case 'failed':
            return status.reason;
        case 'pending':
            return 'pending';
    }
}
