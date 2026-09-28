import { parse_ndjson, type NewlineDeliminatedJson } from './ndjson';
import { req } from './utils.svelte';

// Raw JSON shapes received from the daemon's NDJSON analysis report format.
type RawReportMetadata = {
    analyzers: AnalyzerMetadata[];
    rayhunter?: RayhunterMetadata;
    report_version?: number;
};
type RawEvent = { event_type: string; message: string; analyzer_index?: number };
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
    num_low: number;
    num_medium: number;
    num_high: number;
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
    analyzer_index: number;
};

function is_event_type(s: string): s is EventType {
    return (['Informational', 'Low', 'Medium', 'High'] as const).some((t) => t === s);
}

function parse_event_type(raw: string): EventType {
    if (!is_event_type(raw)) {
        throw `Invalid/unhandled event type: ${raw}`;
    }
    return raw;
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
        const raw_events = row.events ?? [];
        // Detect V3 (dense, each element has analyzer_index) vs V2 (sparse, nulls allowed).
        const first_non_null = raw_events.find((e) => e !== null);
        const is_v3 = first_non_null != null && first_non_null.analyzer_index !== undefined;
        const events: Event[] = raw_events
            .map((e, i): Event | null => {
                if (e === null) {
                    return null;
                }
                return {
                    event_type: parse_event_type(e.event_type),
                    message: e.message,
                    analyzer_index: is_v3 ? (e.analyzer_index ?? i) : i,
                };
            })
            .filter((e): e is Event => e !== null);
        if (events.length > 0) {
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
    let num_low = 0;
    let num_medium = 0;
    let num_high = 0;
    for (const row of rows) {
        if (row.type === AnalysisRowType.Skipped) {
            num_skipped_packets++;
        } else {
            for (const event of row.events) {
                if (event.event_type === 'Informational') {
                    num_informational_logs++;
                } else {
                    num_warnings++;
                    if (event.event_type === 'Low') num_low++;
                    else if (event.event_type === 'Medium') num_medium++;
                    else if (event.event_type === 'High') num_high++;
                }
            }
        }
    }
    return {
        num_warnings,
        num_informational_logs,
        num_skipped_packets,
        num_low,
        num_medium,
        num_high,
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

export type OutdatedAnalyzer = {
    name: string;
    report_version: number;
    current_version: number;
};

// Analyzers whose heuristic has a newer version than the one that produced this report.
// Matched by name, since reports written by older versions don't include the config key.
export function outdated_analyzers(
    report: AnalyzerMetadata[],
    current: AnalyzerMetadata[]
): OutdatedAnalyzer[] {
    const current_versions = new Map(current.map((a) => [a.name, a.version]));
    const outdated: OutdatedAnalyzer[] = [];
    for (const analyzer of report) {
        const current_version = current_versions.get(analyzer.name);
        if (current_version !== undefined && current_version > analyzer.version) {
            outdated.push({
                name: analyzer.name,
                report_version: analyzer.version,
                current_version,
            });
        }
    }
    return outdated;
}
