import { Injectable, signal } from '@angular/core';

export type IndicatorState = 'ok' | 'error';

export interface StatusIndicator {
    id: string;
    label: string;
    state: IndicatorState;
    tooltip?: string | null;
}

export interface StatusSnapshot {
    text: string;
    indicators: StatusIndicator[];
}

@Injectable({
    providedIn: 'root',
})
export class StatusService {
    private _text = signal('');
    private _indicators = signal<StatusIndicator[]>([]);

    public readonly text = this._text.asReadonly();
    public readonly indicators = this._indicators.asReadonly();

    setText(text: string) {
        this._text.set(text);
    }

    /** Adds the indicator, or replaces the one with the same id. */
    setIndicator(indicator: StatusIndicator) {
        this._indicators.update((list) =>
            list.some((i) => i.id === indicator.id)
                ? list.map((i) => (i.id === indicator.id ? indicator : i))
                : [...list, indicator]
        );
    }

    removeIndicator(id: string) {
        this._indicators.update((list) => list.filter((i) => i.id !== id));
    }

    /** Replaces everything at once, e.g. with a snapshot from the backend. */
    apply(snapshot: StatusSnapshot) {
        this._text.set(snapshot.text);
        this._indicators.set(snapshot.indicators);
    }
}
