(function() {var implementors = {};
implementors["serenity"] = [{"text":"impl Freeze for CacheAndHttp","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl Freeze for OpCode","synthetic":true,"types":[]},{"text":"impl Freeze for ApplicationInfo","synthetic":true,"types":[]},{"text":"impl Freeze for BotApplication","synthetic":true,"types":[]},{"text":"impl Freeze for CurrentApplicationInfo","synthetic":true,"types":[]},{"text":"impl Freeze for Team","synthetic":true,"types":[]},{"text":"impl Freeze for TeamMember","synthetic":true,"types":[]},{"text":"impl Freeze for MembershipState","synthetic":true,"types":[]},{"text":"impl Freeze for Attachment","synthetic":true,"types":[]},{"text":"impl&lt;H&gt; Freeze for MessagesIter&lt;H&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;H: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Embed","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedAuthor","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedField","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedFooter","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedImage","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedProvider","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedThumbnail","synthetic":true,"types":[]},{"text":"impl Freeze for EmbedVideo","synthetic":true,"types":[]},{"text":"impl Freeze for GuildChannel","synthetic":true,"types":[]},{"text":"impl Freeze for Message","synthetic":true,"types":[]},{"text":"impl Freeze for MessageReaction","synthetic":true,"types":[]},{"text":"impl Freeze for MessageApplication","synthetic":true,"types":[]},{"text":"impl Freeze for MessageActivity","synthetic":true,"types":[]},{"text":"impl Freeze for MessageReference","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelMention","synthetic":true,"types":[]},{"text":"impl Freeze for MessageFlags","synthetic":true,"types":[]},{"text":"impl Freeze for PrivateChannel","synthetic":true,"types":[]},{"text":"impl Freeze for Reaction","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionConversionError","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelCategory","synthetic":true,"types":[]},{"text":"impl Freeze for PermissionOverwrite","synthetic":true,"types":[]},{"text":"impl Freeze for MessageType","synthetic":true,"types":[]},{"text":"impl Freeze for MessageActivityKind","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionType","synthetic":true,"types":[]},{"text":"impl Freeze for NeverFails","synthetic":true,"types":[]},{"text":"impl Freeze for Channel","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelType","synthetic":true,"types":[]},{"text":"impl Freeze for PermissionOverwriteType","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelCreateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelDeleteEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelPinsUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildBanAddEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildBanRemoveEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildCreateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildDeleteEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildEmojisUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildIntegrationsUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildMemberAddEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildMemberRemoveEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildMemberUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildMembersChunkEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildRoleCreateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildRoleDeleteEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildRoleUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for InviteCreateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for InviteDeleteEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildUnavailableEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GuildUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for MessageCreateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for MessageDeleteBulkEvent","synthetic":true,"types":[]},{"text":"impl Freeze for MessageDeleteEvent","synthetic":true,"types":[]},{"text":"impl Freeze for MessageUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for PresenceUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for PresencesReplaceEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionAddEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionRemoveEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionRemoveAllEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ReadyEvent","synthetic":true,"types":[]},{"text":"impl Freeze for ResumedEvent","synthetic":true,"types":[]},{"text":"impl Freeze for TypingStartEvent","synthetic":true,"types":[]},{"text":"impl Freeze for UnknownEvent","synthetic":true,"types":[]},{"text":"impl Freeze for UserUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for VoiceServerUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for VoiceStateUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for WebhookUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for GatewayEvent","synthetic":true,"types":[]},{"text":"impl Freeze for Event","synthetic":true,"types":[]},{"text":"impl Freeze for EventType","synthetic":true,"types":[]},{"text":"impl Freeze for BotGateway","synthetic":true,"types":[]},{"text":"impl Freeze for Activity","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityAssets","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityFlags","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityParty","synthetic":true,"types":[]},{"text":"impl Freeze for ActivitySecrets","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityEmoji","synthetic":true,"types":[]},{"text":"impl Freeze for Gateway","synthetic":true,"types":[]},{"text":"impl Freeze for ClientStatus","synthetic":true,"types":[]},{"text":"impl Freeze for Presence","synthetic":true,"types":[]},{"text":"impl Freeze for Ready","synthetic":true,"types":[]},{"text":"impl Freeze for SessionStartLimit","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityTimestamps","synthetic":true,"types":[]},{"text":"impl Freeze for ActivityType","synthetic":true,"types":[]},{"text":"impl Freeze for Emoji","synthetic":true,"types":[]},{"text":"impl&lt;H&gt; Freeze for MembersIter&lt;H&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;H: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Integration","synthetic":true,"types":[]},{"text":"impl Freeze for IntegrationAccount","synthetic":true,"types":[]},{"text":"impl Freeze for Member","synthetic":true,"types":[]},{"text":"impl Freeze for PartialMember","synthetic":true,"types":[]},{"text":"impl Freeze for PartialGuild","synthetic":true,"types":[]},{"text":"impl Freeze for Role","synthetic":true,"types":[]},{"text":"impl Freeze for Change","synthetic":true,"types":[]},{"text":"impl Freeze for AuditLogs","synthetic":true,"types":[]},{"text":"impl Freeze for AuditLogEntry","synthetic":true,"types":[]},{"text":"impl Freeze for Options","synthetic":true,"types":[]},{"text":"impl Freeze for Ban","synthetic":true,"types":[]},{"text":"impl Freeze for Guild","synthetic":true,"types":[]},{"text":"impl Freeze for GuildEmbed","synthetic":true,"types":[]},{"text":"impl Freeze for GuildPrune","synthetic":true,"types":[]},{"text":"impl Freeze for GuildInfo","synthetic":true,"types":[]},{"text":"impl Freeze for GuildUnavailable","synthetic":true,"types":[]},{"text":"impl Freeze for Target","synthetic":true,"types":[]},{"text":"impl Freeze for Action","synthetic":true,"types":[]},{"text":"impl Freeze for ActionChannel","synthetic":true,"types":[]},{"text":"impl Freeze for ActionChannelOverwrite","synthetic":true,"types":[]},{"text":"impl Freeze for ActionMember","synthetic":true,"types":[]},{"text":"impl Freeze for ActionRole","synthetic":true,"types":[]},{"text":"impl Freeze for ActionInvite","synthetic":true,"types":[]},{"text":"impl Freeze for ActionWebhook","synthetic":true,"types":[]},{"text":"impl Freeze for ActionEmoji","synthetic":true,"types":[]},{"text":"impl Freeze for ActionMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ActionIntegration","synthetic":true,"types":[]},{"text":"impl Freeze for PremiumTier","synthetic":true,"types":[]},{"text":"impl Freeze for GuildContainer","synthetic":true,"types":[]},{"text":"impl Freeze for GuildStatus","synthetic":true,"types":[]},{"text":"impl Freeze for DefaultMessageNotificationLevel","synthetic":true,"types":[]},{"text":"impl Freeze for ExplicitContentFilter","synthetic":true,"types":[]},{"text":"impl Freeze for MfaLevel","synthetic":true,"types":[]},{"text":"impl Freeze for Region","synthetic":true,"types":[]},{"text":"impl Freeze for VerificationLevel","synthetic":true,"types":[]},{"text":"impl Freeze for ApplicationId","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelId","synthetic":true,"types":[]},{"text":"impl Freeze for EmojiId","synthetic":true,"types":[]},{"text":"impl Freeze for GuildId","synthetic":true,"types":[]},{"text":"impl Freeze for IntegrationId","synthetic":true,"types":[]},{"text":"impl Freeze for MessageId","synthetic":true,"types":[]},{"text":"impl Freeze for RoleId","synthetic":true,"types":[]},{"text":"impl Freeze for UserId","synthetic":true,"types":[]},{"text":"impl Freeze for WebhookId","synthetic":true,"types":[]},{"text":"impl Freeze for AuditLogEntryId","synthetic":true,"types":[]},{"text":"impl Freeze for AttachmentId","synthetic":true,"types":[]},{"text":"impl Freeze for Invite","synthetic":true,"types":[]},{"text":"impl Freeze for InviteUser","synthetic":true,"types":[]},{"text":"impl Freeze for InviteChannel","synthetic":true,"types":[]},{"text":"impl Freeze for InviteGuild","synthetic":true,"types":[]},{"text":"impl Freeze for RichInvite","synthetic":true,"types":[]},{"text":"impl Freeze for EmojiIdentifier","synthetic":true,"types":[]},{"text":"impl Freeze for AffectedComponent","synthetic":true,"types":[]},{"text":"impl Freeze for Incident","synthetic":true,"types":[]},{"text":"impl Freeze for IncidentUpdate","synthetic":true,"types":[]},{"text":"impl Freeze for Maintenance","synthetic":true,"types":[]},{"text":"impl Freeze for UserParseError","synthetic":true,"types":[]},{"text":"impl Freeze for UserIdParseError","synthetic":true,"types":[]},{"text":"impl Freeze for RoleIdParseError","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelIdParseError","synthetic":true,"types":[]},{"text":"impl Freeze for ChannelParseError","synthetic":true,"types":[]},{"text":"impl Freeze for RoleParseError","synthetic":true,"types":[]},{"text":"impl Freeze for IncidentStatus","synthetic":true,"types":[]},{"text":"impl Freeze for Permissions","synthetic":true,"types":[]},{"text":"impl Freeze for CurrentUser","synthetic":true,"types":[]},{"text":"impl Freeze for User","synthetic":true,"types":[]},{"text":"impl Freeze for DefaultAvatar","synthetic":true,"types":[]},{"text":"impl Freeze for OnlineStatus","synthetic":true,"types":[]},{"text":"impl Freeze for VoiceRegion","synthetic":true,"types":[]},{"text":"impl Freeze for VoiceState","synthetic":true,"types":[]},{"text":"impl Freeze for Webhook","synthetic":true,"types":[]},{"text":"impl Freeze for Context","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl Freeze for CreateEmbed","synthetic":true,"types":[]},{"text":"impl Freeze for CreateEmbedAuthor","synthetic":true,"types":[]},{"text":"impl Freeze for CreateEmbedFooter","synthetic":true,"types":[]},{"text":"impl Freeze for Timestamp","synthetic":true,"types":[]},{"text":"impl Freeze for CreateChannel","synthetic":true,"types":[]},{"text":"impl Freeze for CreateInvite","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for CreateMessage&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for CreateAllowedMentions","synthetic":true,"types":[]},{"text":"impl Freeze for EditChannel","synthetic":true,"types":[]},{"text":"impl Freeze for EditGuild","synthetic":true,"types":[]},{"text":"impl Freeze for EditMember","synthetic":true,"types":[]},{"text":"impl Freeze for EditMessage","synthetic":true,"types":[]},{"text":"impl Freeze for EditProfile","synthetic":true,"types":[]},{"text":"impl Freeze for EditRole","synthetic":true,"types":[]},{"text":"impl Freeze for ExecuteWebhook","synthetic":true,"types":[]},{"text":"impl Freeze for GetMessages","synthetic":true,"types":[]},{"text":"impl Freeze for ParseValue","synthetic":true,"types":[]},{"text":"impl Freeze for Settings","synthetic":true,"types":[]},{"text":"impl !Freeze for Cache","synthetic":true,"types":[]},{"text":"impl Freeze for Extras","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for ClientBuilder&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for Client","synthetic":true,"types":[]},{"text":"impl Freeze for ShardManager","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for ShardManagerOptions&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ShardManagerMonitor","synthetic":true,"types":[]},{"text":"impl Freeze for ShardMessenger","synthetic":true,"types":[]},{"text":"impl Freeze for ShardQueuer","synthetic":true,"types":[]},{"text":"impl !Freeze for ShardRunner","synthetic":true,"types":[]},{"text":"impl !Freeze for ShardRunnerOptions","synthetic":true,"types":[]},{"text":"impl Freeze for GatewayIntents","synthetic":true,"types":[]},{"text":"impl Freeze for ShardId","synthetic":true,"types":[]},{"text":"impl Freeze for ShardRunnerInfo","synthetic":true,"types":[]},{"text":"impl Freeze for ShardManagerError","synthetic":true,"types":[]},{"text":"impl Freeze for ShardRunnerMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ChunkGuildFilter","synthetic":true,"types":[]},{"text":"impl Freeze for ShardClientMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ShardManagerMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ShardQueuerMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ShardStageUpdateEvent","synthetic":true,"types":[]},{"text":"impl Freeze for Args","synthetic":true,"types":[]},{"text":"impl&lt;'a, T&gt; Freeze for Iter&lt;'a, T&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for RawArguments&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for Configuration","synthetic":true,"types":[]},{"text":"impl Freeze for WithWhiteSpace","synthetic":true,"types":[]},{"text":"impl Freeze for Check","synthetic":true,"types":[]},{"text":"impl Freeze for CommandOptions","synthetic":true,"types":[]},{"text":"impl Freeze for Command","synthetic":true,"types":[]},{"text":"impl Freeze for HelpCommand","synthetic":true,"types":[]},{"text":"impl Freeze for HelpOptions","synthetic":true,"types":[]},{"text":"impl Freeze for GroupOptions","synthetic":true,"types":[]},{"text":"impl Freeze for CommandGroup","synthetic":true,"types":[]},{"text":"impl Freeze for BucketBuilder","synthetic":true,"types":[]},{"text":"impl !Freeze for StandardFramework","synthetic":true,"types":[]},{"text":"impl Freeze for Delimiter","synthetic":true,"types":[]},{"text":"impl&lt;E&gt; Freeze for Error&lt;E&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;E: Freeze,&nbsp;</span>","synthetic":true,"types":[]},{"text":"impl Freeze for Reason","synthetic":true,"types":[]},{"text":"impl Freeze for CheckResult","synthetic":true,"types":[]},{"text":"impl Freeze for OnlyIn","synthetic":true,"types":[]},{"text":"impl Freeze for HelpBehaviour","synthetic":true,"types":[]},{"text":"impl Freeze for DispatchError","synthetic":true,"types":[]},{"text":"impl Freeze for GroupCommandsPair","synthetic":true,"types":[]},{"text":"impl Freeze for SuggestedCommandName","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for Command&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for Suggestions","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for CustomisedHelpData&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl !Freeze for Shard","synthetic":true,"types":[]},{"text":"impl Freeze for ConnectionStage","synthetic":true,"types":[]},{"text":"impl Freeze for InterMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ShardAction","synthetic":true,"types":[]},{"text":"impl Freeze for ReconnectType","synthetic":true,"types":[]},{"text":"impl Freeze for LightMethod","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for AttachmentType&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for GuildPagination","synthetic":true,"types":[]},{"text":"impl Freeze for Http","synthetic":true,"types":[]},{"text":"impl Freeze for DiscordJsonError","synthetic":true,"types":[]},{"text":"impl Freeze for ErrorResponse","synthetic":true,"types":[]},{"text":"impl Freeze for Error","synthetic":true,"types":[]},{"text":"impl Freeze for Ratelimiter","synthetic":true,"types":[]},{"text":"impl Freeze for Ratelimit","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for RatelimitedRequest&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for RequestBuilder&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for Request&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for Route","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for RouteInfo&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for Typing","synthetic":true,"types":[]},{"text":"impl Freeze for Colour","synthetic":true,"types":[]},{"text":"impl Freeze for Content","synthetic":true,"types":[]},{"text":"impl Freeze for MessageBuilder","synthetic":true,"types":[]},{"text":"impl Freeze for CustomMessage","synthetic":true,"types":[]},{"text":"impl Freeze for ContentSafeOptions","synthetic":true,"types":[]},{"text":"impl Freeze for ContentModifier","synthetic":true,"types":[]},{"text":"impl Freeze for MessageFilter","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for MessageCollectorBuilder&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for CollectReply&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for MessageCollector","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionFilter","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for ReactionCollectorBuilder&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl&lt;'a&gt; Freeze for CollectReaction&lt;'a&gt;","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionCollector","synthetic":true,"types":[]},{"text":"impl Freeze for ReactionAction","synthetic":true,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()