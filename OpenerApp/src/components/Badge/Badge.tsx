import { FC } from "react";

export enum BadgeState {
    Waiting,
    Checking,
    Accepted,
    Rejected
}

export type BadgeProps = {
    state: BadgeState
};

const state_dependant_classes = {
    main_badge: [
        "top-[135vh] left-1/2",
        "top-1/2 left-1/2",
        "top-1/2 left-1/2 animate-[badge-accept_2s_ease-in_1_both]",
        "top-1/2 left-1/2 animate-[badge-reject_2s_ease-in_1_both]",
    ],
    emblem: [
        "bg-ph-yellow",
        "bg-ph-yellow",
        "bg-green-500",
        "bg-red-500"
    ],
    spinner: [
        "border-ph-yellow",
        "border-ph-yellow border-t-transparent",
        "border-green-500",
        "border-red-500"
    ]
}

export const Badge: FC<BadgeProps> = ({  
    state
}) => {
    return (
        <div className={`transition-all duration-1000 fixed -translate-x-1/2 -translate-y-1/2 w-[332px] h-[472px] border-ph-yellow border-2 rounded-xl backdrop-blur-xl ${state_dependant_classes.main_badge[state]}`}>
            <div className='w-full h-full bg-[url("noise.svg")] rounded-lg opacity-[15%]'></div>
           
            <div 
                className={`transition-all absolute -translate-x-1/2 -translate-y-1/2 top-1/2 left-1/2 w-[200px] h-[200px] ${state_dependant_classes.emblem[state]}`}
                style={{
                    maskImage: "url('passport-emblem.svg')",
                    maskPosition: "center"
                }}
            />

            <div className={`animate-[safe-spin_1.5s_linear_infinite] transition-all absolute -translate-x-1/2 -translate-y-1/2 top-1/2 left-1/2 w-[130px] h-[130px] border-2 rounded-full ${state_dependant_classes.spinner[state]}`}/>
        
        </div>
    );
};