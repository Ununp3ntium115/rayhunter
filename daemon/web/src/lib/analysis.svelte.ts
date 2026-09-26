import { parse_ndjson, type NewlineDeliminatedJson } from './ndjson';
import { req } from './utils.svelte';

// Raw JSON shapes received from the daemon's NDJSON analysis report format.
type RawReportMetadata = {
    analyzers: AnalyzerMetadata[];
    rayhunter?: RayhunterMetadata;
    report_version?: number;
};
type RawEvent = { event_type: string; message: string };
type RawAnalysisRow = {
    skipped_message_reason?: string;
    events?: (RawEvent | null)[];
    packet_timestamp: string;
};

export type AnalysisReport = {
    metadata: ReportMetadata;
    rows: AnalysisRow[];
    statistics: ReportStatistics;
};

export type ReportStatistics = {
    num_warnings: number;
    num_informational_logs: number;
    num_skipped_packets: number;
};

export class ReportMetadata {
    public analyzers: AnalyzerMetadata[];
    public rayhunter: RayhunterMetadata;
    public report_version: number;

    constructor(ndjson: unknown) {
        // SAFETY: ndjson is report_json[0] from the daemon's NDJSON report — always the metadata line.
        const raw = ndjson as RawReportMetadata;
        this.analyzers = raw.analyzers;
        // SAFETY: rayhunter field is always present in daemon NDJSON output.
        this.rayhunter = raw.rayhunter as RayhunterMetadata;
        this.report_version = raw.report_version ?? 2;
    }
}

export type RayhunterMetadata = {
    rayhunter_version: string;
    system_os: string;
    arch: string;
};

export type AnalyzerMetadata = {
    name: string;
    description: string;
    version: number;
};

export type AnalysisRow = SkippedPacket | PacketAnalysis;
export enum AnalysisRowType {
    Skipped,
    Analysis,
}

export type SkippedPacket = {
    type: AnalysisRowType.Skipped;
    reason: string;
};

export type PacketAnalysis = {
    type: AnalysisRowType.Analysis;
    packet_timestamp: Date;
    events: Event[];
};

export type EventType = 'Informational' | 'Low' | 'Medium' | 'High';

export type Event = {
    event_type: EventType;
    message: string;
} | null;

function is_event_type(s: string): s is EventType {
    return (['Informational', 'Low', 'Medium', 'High'] as const).some((t) => t === s);
}

function get_event(event_json: unknown): Event {
    // SAFETY: event_json is a non-null element of a daemon NDJSON events array; null is filtered before this call.
    const raw = event_json as RawEvent;
    const event_type = raw.event_type;
    if (!is_event_type(event_type)) {
        throw `Invalid/unhandled event type: ${event_type}`;
    }

    return { event_type, message: raw.message };
}

function get_rows(row_jsons: unknown[]): AnalysisRow[] {
    const rows: AnalysisRow[] = [];
    for (const row_json of row_jsons) {
        // SAFETY: row_jsons elements are objects from the daemon's NDJSON analysis rows.
        const row = row_json as RawAnalysisRow;
        if (row.skipped_message_reason) {
            rows.push({
                type: AnalysisRowType.Skipped,
                reason: row.skipped_message_reason,
            });
        }
        const events: Event[] = (row.events ?? []).map((event_json): Event | null => {
            if (event_json === null) {
                return null;
            } else {
                return get_event(event_json);
            }
        });
        if (events.some((event) => event !== null)) {
            rows.push({
                type: AnalysisRowType.Analysis,
                packet_timestamp: new Date(row.packet_timestamp),
                events,
            });
        }
    }
    return rows;
}

function get_report_stats(rows: AnalysisRow[]): ReportStatistics {
    let num_warnings = 0;
    let num_informational_logs = 0;
    let num_skipped_packets = 0;
    for (const row of rows) {
        if (row.type === AnalysisRowType.Skipped) {
            num_skipped_packets++;
        } else {
            for (const event of row.events) {
                if (event !== null) {
                    if (event.event_type === 'Informational') {
                        num_informational_logs++;
                    } else {
                        num_warnings++;
                    }
                }
            }
        }
    }
    return {
        num_warnings,
        num_informational_logs,
        num_skipped_packets,
    };
}

export function parse_finished_report(report_json: NewlineDeliminatedJson): AnalysisReport {
    const metadata = new ReportMetadata(report_json[0]);
    const rows = get_rows(report_json.slice(1));
    const statistics = get_report_stats(rows);
    return {
        statistics,
        metadata,
        rows,
    };
}

export async function get_report(name: string): Promise<AnalysisReport> {
    const report_json = parse_ndjson(await req('GET', `/api/analysis-report/${name}`));
    return parse_finished_report(report_json);
}
