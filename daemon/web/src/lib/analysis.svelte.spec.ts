import { describe, it, expect } from 'vitest';
import { AnalysisRowType, parse_finished_report } from './analysis.svelte';
import { type NewlineDeliminatedJson } from './ndjson';

const SAMPLE_V2_REPORT_NDJSON: NewlineDeliminatedJson = [
    {
        analyzers: [
            {
                name: 'Analyzer 1',
                description: 'A first analyzer',
                version: 2,
            },
            {
                name: 'Analyzer 2',
                description: 'A second analyzer',
                version: 2,
            },
        ],
        report_version: 2,
    },
    {
        packet_timestamp: '2024-08-19T03:33:54.318Z',
        skipped_message_reason: 'The reason why the message was skipped',
        events: [
            {
                event_type: 'Low',
                message: 'A warning from a skipped packet',
            },
            null,
        ],
    },
    {
        packet_timestamp: '2024-08-19T03:33:54.318Z',
        events: [
            null,
            {
                event_type: 'Low',
                message: 'Something nasty happened',
            },
        ],
    },
];

const SAMPLE_V3_REPORT_NDJSON: NewlineDeliminatedJson = [
    {
        analyzers: [
            {
                name: 'Analyzer 1',
                description: 'A first analyzer',
                version: 3,
            },
            {
                name: 'Analyzer 2',
                description: 'A second analyzer',
                version: 3,
            },
        ],
        report_version: 3,
    },
    {
        packet_timestamp: '2024-08-19T03:33:54.318Z',
        skipped_message_reason: 'The reason why the message was skipped',
        events: [
            {
                event_type: 'Low',
                message: 'A warning from a skipped packet',
                analyzer_index: 0,
            },
        ],
    },
    {
        packet_timestamp: '2024-08-19T03:33:54.318Z',
        events: [
            {
                event_type: 'Low',
                message: 'Something nasty happened',
                analyzer_index: 1,
            },
        ],
    },
];

describe('analysis report parsing', () => {
    it('parses v2 example analysis (backwards compat)', () => {
        const report = parse_finished_report(SAMPLE_V2_REPORT_NDJSON);
        expect(report.metadata.report_version).toEqual(2);
        expect(report.metadata.analyzers).toEqual([
            {
                name: 'Analyzer 1',
                description: 'A first analyzer',
                version: 2,
            },
            {
                name: 'Analyzer 2',
                description: 'A second analyzer',
                version: 2,
            },
        ]);
        expect(report.rows).toHaveLength(3);
        expect(report.rows[0].type).toBe(AnalysisRowType.Skipped);
        if (report.rows[1].type === AnalysisRowType.Analysis) {
            const row = report.rows[1];
            // V2 null at index 1 is dropped; only the non-null event survives
            expect(row.events).toHaveLength(1);
            expect(row.events[0].message).toEqual('A warning from a skipped packet');
            expect(row.events[0].analyzer_index).toEqual(0);
        } else {
            throw 'wrong row type';
        }
        if (report.rows[2].type === AnalysisRowType.Analysis) {
            const row = report.rows[2];
            // V2 null at index 0 is dropped; only the event at position 1 survives
            expect(row.events).toHaveLength(1);
            const event = row.events[0];
            const expected_timestamp = new Date('2024-08-19T03:33:54.318Z');
            expect(row.packet_timestamp.getTime()).toEqual(expected_timestamp.getTime());
            expect(event.event_type).toEqual('Low');
            expect(event.analyzer_index).toEqual(1);
        } else {
            throw 'wrong row type';
        }
        expect(report.statistics.num_warnings).toEqual(2);
        expect(report.statistics.num_skipped_packets).toEqual(1);
    });

    it('parses v3 example analysis', () => {
        const report = parse_finished_report(SAMPLE_V3_REPORT_NDJSON);
        expect(report.metadata.report_version).toEqual(3);
        expect(report.rows).toHaveLength(3);
        expect(report.rows[0].type).toBe(AnalysisRowType.Skipped);
        if (report.rows[1].type === AnalysisRowType.Analysis) {
            const row = report.rows[1];
            expect(row.events).toHaveLength(1);
            expect(row.events[0].message).toEqual('A warning from a skipped packet');
            expect(row.events[0].analyzer_index).toEqual(0);
        } else {
            throw 'wrong row type';
        }
        if (report.rows[2].type === AnalysisRowType.Analysis) {
            const row = report.rows[2];
            expect(row.events).toHaveLength(1);
            // V3: analyzer_index is explicit; event at index 1 not position 0
            expect(row.events[0].analyzer_index).toEqual(1);
            expect(row.events[0].event_type).toEqual('Low');
        } else {
            throw 'wrong row type';
        }
        expect(report.statistics.num_warnings).toEqual(2);
        expect(report.statistics.num_skipped_packets).toEqual(1);
    });
});
