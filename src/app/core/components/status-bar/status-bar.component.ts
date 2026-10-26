import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { IndicatorState, StatusIndicator } from '../../services/status.service';

/** Marker colors and screen-reader text per indicator state. */
const STATE_STYLES: Record<IndicatorState, { fill: string; border: string; label: string }> = {
    ok: { fill: 'var(--c-ok)', border: 'var(--c-parchment)', label: 'OK' },
    warning: { fill: 'var(--c-warn)', border: 'var(--c-parchment)', label: 'Warning' },
    error: { fill: 'var(--c-blood-bright)', border: 'var(--c-gold-bright)', label: 'Error' },
};

@Component({
    selector: 'app-status-bar',
    templateUrl: './status-bar.component.html',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StatusBarComponent {
    readonly text = input('');
    readonly indicators = input<StatusIndicator[]>([]);
    protected readonly styles = STATE_STYLES;
}
