import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { StatusIndicator } from '../../services/status.service';

@Component({
    selector: 'app-status-bar',
    templateUrl: './status-bar.component.html',
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StatusBarComponent {
    readonly text = input('');
    readonly indicators = input<StatusIndicator[]>([]);
}
